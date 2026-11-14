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
            // Until backlog A1, everyone is right-handed.
            assert_eq!((e.bats(), e.throws()), (bb_schema::Hand::Right, bb_schema::Hand::Right));
        }
    }
}
