//! Three whole games scored by hand (review P2-4), pinned against the box score.
//!
//! The logs are saved from seeds 3, 7 and 1001 (`examples/write_log`), so these cases do not move
//! when the simulation does. Each play was read from the raw events of the log (`examples/dump_plays`:
//! the outs, advances and runs the rules engine wrote, with where the ball was fielded and the
//! logged throws), and ruled by Rule 9 by hand, not by the scorer. Rulings that took a judgement:
//!
//! - A batter put out at the next base stretching a hit is credited with the last base he
//!   reached (9.06(c)): seed 3 Fujita single, Brooks single, Ishikawa triple (out at home);
//!   seed 7 Davis, Thompson (doubles, out at third).
//! - A runner forced out by the batter's becoming a runner, or an infielder's putout of a runner
//!   advancing one base, is a fielder's choice, not a hit, even if an outfielder fielded it or the
//!   batter would have beaten the throw (9.05(b)(1), (3)): seed 3 Jenkins; seed 7 Hernandez, Morgan;
//!   seed 1001 Castillo (with the run an RBI: a fielder's choice that scores a run, 9.04(a)).
//! - A throw at a runner ahead, made while the batter was still running and not enough to have
//!   retired him (the ordinary throw would have been late), leaves the hit a hit (9.05(a)(6)).
//! - Infield grounders the batter beat although the fielder threw at first, with no misplay in the
//!   record, are hits (the sim has no errors): seed 3 Hayes; seed 1001 Bennett, Santana.
//! - Double plays by force: no RBI is in question here; a run on a force out that is not a double
//!   play is an RBI (seed 3 Price).
use bb_stats::box_score;

/// (family name, AB, R, H, 2B, 3B, HR, RBI, BB, K)
type Row = (&'static str, u32, u32, u32, u32, u32, u32, u32, u32, u32);

const GAMES: &[(u32, &[u8], &[Row])] = &[
    (3, include_bytes!("fixtures/game_3.bblog"), &[("Nakamura", 2, 0, 0, 0, 0, 0, 0, 2, 2), ("Walker", 3, 1, 2, 1, 0, 0, 0, 1, 0), ("Anderson", 3, 0, 1, 0, 0, 0, 0, 1, 1), ("Jenkins", 4, 0, 0, 0, 0, 0, 0, 0, 0), ("Kobayashi", 4, 0, 1, 0, 0, 0, 0, 0, 0), ("Fujita", 4, 0, 1, 0, 0, 0, 0, 0, 2), ("Hayes", 3, 0, 1, 0, 0, 0, 0, 1, 1), ("Ishikawa", 3, 0, 1, 0, 1, 0, 1, 0, 0), ("Lewis", 4, 0, 0, 0, 0, 0, 0, 0, 3), ("Miller", 3, 2, 2, 1, 1, 0, 0, 0, 0), ("Young", 4, 0, 0, 0, 0, 0, 0, 0, 0), ("Brooks", 3, 0, 2, 0, 0, 0, 1, 0, 0), ("Espinoza", 4, 0, 0, 0, 0, 0, 0, 0, 1), ("Price", 3, 0, 0, 0, 0, 0, 1, 0, 0), ("Ward", 2, 1, 1, 0, 0, 1, 1, 1, 0), ("Morgan", 2, 0, 1, 0, 0, 0, 0, 1, 0), ("Cook", 3, 0, 0, 0, 0, 0, 0, 0, 2), ("Russell", 3, 0, 0, 0, 0, 0, 0, 0, 1)]),
    (7, include_bytes!("fixtures/game_7.bblog"), &[("Kobayashi", 4, 0, 1, 0, 0, 0, 0, 1, 1), ("Thompson", 4, 0, 2, 2, 0, 0, 0, 0, 0), ("Davis", 4, 1, 2, 1, 0, 0, 0, 0, 1), ("Hernandez", 3, 1, 0, 0, 0, 0, 0, 1, 0), ("Hwang", 4, 0, 1, 1, 0, 0, 0, 0, 0), ("Phillips", 3, 0, 2, 0, 0, 0, 0, 1, 0), ("Cruz", 4, 0, 0, 0, 0, 0, 0, 0, 2), ("Nunez", 4, 1, 2, 1, 0, 0, 0, 0, 0), ("Harris", 3, 0, 1, 0, 0, 0, 0, 1, 2), ("Carter", 4, 0, 2, 0, 0, 0, 1, 0, 0), ("Flores", 4, 1, 4, 0, 1, 0, 1, 0, 0), ("Cooper", 4, 0, 2, 0, 0, 0, 0, 0, 0), ("Baker", 4, 1, 1, 0, 0, 1, 2, 0, 1), ("Roberts", 4, 0, 2, 0, 0, 0, 0, 0, 1), ("Hayes", 4, 0, 2, 0, 0, 0, 1, 0, 1), ("Hayashi", 4, 0, 1, 1, 0, 0, 0, 0, 0), ("Morgan", 4, 0, 0, 0, 0, 0, 0, 0, 0), ("Fujita", 4, 0, 0, 0, 0, 0, 0, 0, 2)]),
    (1001, include_bytes!("fixtures/game_1001.bblog"), &[("Matsumoto", 5, 2, 3, 1, 0, 1, 2, 0, 2), ("Hernandez", 5, 1, 2, 1, 0, 0, 1, 0, 0), ("Stewart", 5, 1, 2, 1, 0, 0, 1, 0, 1), ("Bennett", 3, 1, 2, 0, 0, 0, 0, 1, 0), ("Guerrero", 5, 0, 1, 0, 0, 0, 0, 0, 0), ("Allen", 4, 0, 1, 0, 0, 0, 0, 0, 0), ("Anderson", 3, 0, 2, 1, 0, 0, 1, 1, 1), ("Wilson", 3, 0, 0, 0, 0, 0, 1, 1, 1), ("Cooper", 4, 1, 1, 0, 0, 1, 1, 0, 2), ("Castillo", 4, 0, 1, 0, 0, 0, 1, 0, 1), ("Jenkins", 4, 0, 0, 0, 0, 0, 0, 0, 0), ("Morgan", 4, 0, 1, 1, 0, 0, 0, 0, 1), ("Wright", 4, 0, 0, 0, 0, 0, 0, 0, 1), ("Bell", 4, 0, 1, 1, 0, 0, 0, 0, 1), ("Santana", 4, 0, 1, 0, 0, 0, 0, 0, 1), ("Sanchez", 4, 1, 2, 1, 1, 0, 0, 0, 1), ("Rogers", 3, 1, 1, 1, 0, 0, 0, 1, 0), ("Hughes", 4, 0, 0, 0, 0, 0, 0, 0, 2)]),
];

#[test]
fn three_games_scored_by_hand_match_the_box_score() {
    for &(seed, log, rows) in GAMES {
        let b = box_score(log).unwrap();
        let mut seen = 0;
        for side in 0..2 {
            for l in &b.batting[side] {
                let want = rows.iter().find(|r| r.0 == l.family).unwrap_or_else(|| panic!("seed {seed}: no hand line for {}", l.family));
                let got = (l.family.as_str(), l.ab, l.runs, l.h, l.doubles, l.triples, l.hr, l.rbi, l.bb, l.k);
                assert_eq!(got, *want, "seed {seed}, {}", l.family);
                seen += 1;
            }
        }
        assert_eq!(seen, rows.len(), "seed {seed}: every hand-scored batter is in the box score");
        assert_eq!(b.teams.iter().map(|t| t.undetermined).sum::<u32>(), 0);
    }
}
