//! Rosters in a real game (data-formats §2.4): the names a viewer reads belong to the batter
//! the batting order says is up, and the log's header carries the same players.

use bb_game::describe::describe;
use bb_game::game::Game;
use bb_game::roster::Rosters;
use bb_rules::{Event, Half};

/// The batter named on each plate appearance is the one the batting order puts up: each side
/// goes through its nine in order, again and again, whatever happens in between. The
/// expectation is counted here, not read from the rules state.
#[test]
fn play_by_play_names_the_batter_the_order_puts_up() {
    for seed in [7u64, 21] {
        let rosters = Rosters::generate(seed);
        let mut g = Game::new(seed, true);
        assert_eq!(g.rosters, rosters);
        let mut up = [0usize; 2];
        let (mut named, mut labelled) = (0, 0);
        loop {
            let side = if g.st.half == Half::Top { 0 } else { 1 };
            let Some(step) = g.step_pitch() else { break };
            let team = &rosters.teams[side];
            let batter = team.batting_order()[up[side] % 9];
            assert_eq!(step.batter, team.bat_agent(batter), "seed {seed} play {}", step.play);
            // The lab label names him and his position on every pitch.
            assert!(step.viz.label.contains(&format!("#{} {} {}:", up[side] % 9 + 1, batter.position_code(), batter.family)), "{}", step.viz.label);
            labelled += 1;
            let text = describe(step.record.call, step.record.batted.map(|b| b.1.to_degrees()), &step.rules_events, &step.facts, Some(&g.rosters));
            let plain = describe(step.record.call, step.record.batted.map(|b| b.1.to_degrees()), &step.rules_events, &step.facts, None);
            if step.rules_events.iter().any(|e| matches!(e, Event::PlateAppearanceEnded { .. })) {
                assert_eq!(text, format!("{}: {plain}", batter.family), "seed {seed} play {}", step.play);
                // The caption must fit the C ABI's summary buffer (96 bytes with its NUL).
                assert!(text.len() < 96 && text.is_ascii(), "{text}");
                named += 1;
                up[side] += 1;
            } else {
                // Mid-count calls stay bare ("Ball", "Foul").
                assert_eq!(text, plain);
            }
        }
        // A full game: at least 27 outs a side less the home ninth, so 51 plate appearances.
        assert!(named >= 51 && labelled > named, "seed {seed}: {named} plate appearances named");
    }
}

/// The event log's header carries both rosters, field for field.
#[test]
fn the_log_header_carries_the_rosters() {
    let seed = 7;
    let g = bb_game::game::simulate_game(seed);
    let v = bb_schema::log::read(&g.log).expect("log verifies");
    let cfg = v.header.config().expect("the header has a game config");
    assert_eq!(cfg.game_seed(), seed);
    let rosters = Rosters::generate(seed);
    for (side, team) in [cfg.away().unwrap(), cfg.home().unwrap()].into_iter().enumerate() {
        let want = &rosters.teams[side];
        assert_eq!(team.name(), Some(want.name));
        assert_eq!(team.starting_pitcher(), ((side as u16) << 8) | 0x81);
        let entries = team.roster().unwrap();
        assert_eq!(entries.len(), 9);
        for (e, p) in entries.iter().zip(&want.players) {
            assert_eq!(e.name().unwrap(), format!("{} {}", p.given, p.family));
            assert_eq!(e.family_name(), Some(p.family));
            assert_eq!((e.player_id(), e.uniform_number(), e.position().0, e.batting_order()), (p.player_id, p.number, p.position, p.batting_order));
            assert_eq!((e.agent_id(), e.field_agent_id()), (((side as u16) << 8) | p.batting_order as u16, ((side as u16) << 8) | 0x80 | p.position as u16));
            assert_eq!(e.name_origin(), p.origin as u8);
            let bats = match p.bats {
                bb_observe::Bats::Left => bb_schema::Hand::Left,
                bb_observe::Bats::Right => bb_schema::Hand::Right,
                bb_observe::Bats::Switch => bb_schema::Hand::Switch,
            };
            let throws = if p.throws == bb_observe::Hand::Left { bb_schema::Hand::Left } else { bb_schema::Hand::Right };
            assert_eq!((e.bats(), e.throws()), (bats, throws));
        }
    }
}

/// Hands on the field (owner backlog A1): every pitch leaves the hand of the fielding side's
/// pitcher (a right-hander lets go on the third-base side of the rubber, x < 0; a left-hander
/// on the first-base side), and the batter stands on his own side of the plate (a
/// right-handed batter in the third-base-side box, x < 0), a switch hitter on the side
/// opposite the pitcher's hand. Seeds chosen so that both teams' pitchers' hands, and a
/// switch hitter, are seen.
#[test]
fn pitchers_and_batters_play_from_their_own_sides() {
    use bb_observe::{Bats, Hand};
    let (mut lhp, mut rhp, mut lhb, mut rhb, mut switch_l, mut switch_r) = (0, 0, 0, 0, 0, 0);
    for seed in 0..12u64 {
        let rosters = Rosters::generate(seed);
        let mut g = Game::new(seed, true);
        let mut n = 0;
        loop {
            let side = if g.st.half == Half::Top { 0 } else { 1 };
            let Some(step) = g.step_pitch() else { break };
            let throws = rosters.teams[1 - side].players[0].throws;
            let x_release = step.record.release.r.x;
            assert_eq!(x_release > 0.0, throws == Hand::Left, "seed {seed} play {}: {throws:?} released at x {x_release:.2}", step.play);
            assert!(x_release.abs() > 0.2);
            let batter = rosters.player(step.batter).unwrap();
            let stands = match (batter.bats, throws) {
                (Bats::Left, _) | (Bats::Switch, Hand::Right) => Hand::Left,
                (Bats::Right, _) | (Bats::Switch, Hand::Left) => Hand::Right,
            };
            let x_box = step.viz.standing.iter().find(|(id, _)| *id == step.batter).expect("the batter stands somewhere").1.x;
            assert_eq!(x_box > 0.0, stands == Hand::Left, "seed {seed} play {}: {:?} against {throws:?} stood at x {x_box:.2}", step.play, batter.bats);
            assert!(x_box.abs() > 0.5);
            match throws {
                Hand::Left => lhp += 1,
                Hand::Right => rhp += 1,
            }
            match stands {
                Hand::Left => lhb += 1,
                Hand::Right => rhb += 1,
            }
            if batter.bats == Bats::Switch {
                if stands == Hand::Left { switch_l += 1 } else { switch_r += 1 }
            }
            n += 1;
            // The first three innings or so are enough to see every batter.
            if n > 120 {
                break;
            }
        }
    }
    assert!(lhp > 100 && rhp > 100 && lhb > 100 && rhb > 100, "{lhp} {rhp} {lhb} {rhb}");
    assert!(switch_l > 10 && switch_r > 10, "switch hitters batted left {switch_l} and right {switch_r} times");
}
