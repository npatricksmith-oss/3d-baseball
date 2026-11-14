//! Game rosters: who the players are (data-formats §2.4, owner backlog A2). Identity only:
//! names, uniform numbers, positions, the batting order, and the handedness slot backlog A1
//! fills. Nothing here gives a player any ability, and nothing in a play reads it.
//!
//! Until the league generator exists (M7), a roster is generated from the game seed alone.

use bb_math::{KeyedRng, Purpose, RngKey};
use bb_observe::{Bats, Hand};
use bb_rules::AgentId;

/// Which pool a name came from, kept so a player's look can match his name later (L2).
/// The values are stored in the event log (`RosterEntry.name_origin`): append-only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum NameOrigin {
    English = 1,
    Spanish = 2,
    Japanese = 3,
    Korean = 4,
}

/// One pool of names: given names and family names that go together.
pub struct NamePool {
    pub origin: NameOrigin,
    /// Share of players named from this pool [E] (data-formats §2.4).
    pub weight: f64,
    pub given: &'static [&'static str],
    pub family: &'static [&'static str],
}

/// The name pools [E]. **Editing or reordering a list changes every generated roster.**
/// ASCII only (data-formats §2.4). No family name appears in two pools (tested).
pub const NAME_POOLS: [NamePool; 4] = [
    NamePool {
        origin: NameOrigin::English,
        weight: 0.62,
        given: &[
            "Aaron", "Adam", "Andrew", "Austin", "Ben", "Brandon", "Brett", "Caleb", "Chris", "Cody", "Cole", "Connor", "Daniel", "David", "Derek", "Dylan", "Eric", "Ethan", "Evan",
            "Garrett", "Grant", "Hunter", "Ian", "Jack", "Jake", "James", "Jason", "Jordan", "Josh", "Justin", "Kevin", "Kyle", "Logan", "Luke", "Marcus", "Matt", "Mike", "Nate",
            "Nick", "Owen", "Paul", "Ryan", "Sam", "Sean", "Trevor", "Tyler", "Will", "Zach",
        ],
        family: &[
            "Adams", "Allen", "Anderson", "Bailey", "Baker", "Bell", "Bennett", "Brooks", "Brown", "Campbell", "Carter", "Clark", "Collins", "Cook", "Cooper", "Davis", "Edwards",
            "Evans", "Fisher", "Foster", "Gray", "Green", "Hall", "Harris", "Hayes", "Hill", "Howard", "Hughes", "Jackson", "Jenkins", "Johnson", "Kelly", "King", "Lewis", "Marshall",
            "Miller", "Mitchell", "Moore", "Morgan", "Murphy", "Nelson", "Parker", "Perry", "Phillips", "Porter", "Price", "Reed", "Roberts", "Rogers", "Russell", "Sanders", "Scott",
            "Stewart", "Sullivan", "Taylor", "Thompson", "Turner", "Walker", "Ward", "Watson", "White", "Wilson", "Wood", "Wright", "Young",
        ],
    },
    NamePool {
        origin: NameOrigin::Spanish,
        weight: 0.28,
        given: &[
            "Alejandro", "Andres", "Angel", "Carlos", "Cesar", "Diego", "Eduardo", "Emilio", "Enrique", "Felix", "Fernando", "Francisco", "Gabriel", "Hector", "Ivan", "Javier",
            "Jesus", "Jorge", "Jose", "Juan", "Julio", "Luis", "Manuel", "Marco", "Mario", "Miguel", "Pablo", "Pedro", "Rafael", "Ramon", "Raul", "Ricardo", "Roberto", "Santiago",
            "Victor",
        ],
        family: &[
            "Acosta", "Alvarez", "Castillo", "Castro", "Cruz", "Delgado", "Diaz", "Dominguez", "Espinoza", "Fernandez", "Flores", "Garcia", "Gomez", "Gonzalez", "Guerrero",
            "Gutierrez", "Hernandez", "Jimenez", "Lopez", "Marquez", "Martinez", "Medina", "Mendoza", "Molina", "Morales", "Navarro", "Nunez", "Ortiz", "Pena", "Perez", "Ramirez",
            "Reyes", "Rivera", "Rodriguez", "Rojas", "Romero", "Sanchez", "Santana", "Soto", "Suarez", "Torres", "Valdez", "Vargas", "Vasquez",
        ],
    },
    NamePool {
        origin: NameOrigin::Japanese,
        weight: 0.06,
        given: &["Daiki", "Haruto", "Hiroshi", "Kazuki", "Kenji", "Koji", "Makoto", "Masato", "Naoki", "Ren", "Ryo", "Shota", "Sota", "Takumi", "Yuki", "Yuto"],
        family: &[
            "Fujita", "Hayashi", "Inoue", "Ishikawa", "Ito", "Kato", "Kimura", "Kobayashi", "Matsumoto", "Mori", "Nakamura", "Ogawa", "Okada", "Saito", "Sasaki", "Shimizu",
            "Takahashi", "Tanaka", "Watanabe", "Yamada", "Yamamoto", "Yoshida",
        ],
    },
    NamePool {
        origin: NameOrigin::Korean,
        weight: 0.04,
        given: &["Dong-hyun", "Hyun-woo", "Jae-won", "Ji-ho", "Ji-hoon", "Jun-seo", "Min-jun", "Sang-woo", "Seung-min", "Sung-ho", "Tae-yang", "Woo-jin"],
        family: &["Ahn", "Bae", "Choi", "Hwang", "Jang", "Jeon", "Jung", "Kang", "Kwon", "Lim", "Moon", "Oh", "Seo", "Shin", "Song", "Yoon"],
    },
];

/// One player's identity for a game.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    pub player_id: u32,
    pub given: &'static str,
    pub family: &'static str,
    pub origin: NameOrigin,
    /// Uniform number, 1–99, unique within the team.
    pub number: u8,
    /// Fielding position in the scorer's numbering, 1–9 (data-formats §2.1).
    pub position: u8,
    /// Place in the batting order, 1–9.
    pub batting_order: u8,
    /// The side he bats from; a switch hitter takes the side opposite the pitcher's hand.
    pub bats: Bats,
    /// The hand he throws with; his glove is on the other.
    pub throws: Hand,
}

impl Player {
    /// "Given Family".
    pub fn full_name(&self) -> String {
        format!("{} {}", self.given, self.family)
    }

    /// The scorer's abbreviation of his position ("SS").
    pub fn position_code(&self) -> &'static str {
        POSITION_CODE[self.position as usize - 1]
    }
}

/// Share of players at each position (1–9) who throw left-handed [E]. Catchers, second and
/// third basemen and shortstops throw right: a left-hander there has to turn his body to
/// throw across the infield, so teams do not play one. About a quarter of pitchers and
/// outfielders and a third of first basemen throw left.
pub const THROWS_LEFT: [f64; 9] = [0.27, 0.0, 0.35, 0.0, 0.0, 0.0, 0.25, 0.20, 0.25];

/// The batting side given the throwing hand, as shares (left, switch; the rest bat right)
/// [E]. Most players bat from the side they throw from, but many right-handed throwers bat
/// left or both ways, and few left-handed throwers bat right. With `THROWS_LEFT` this puts
/// about 32% of position players on the left side, 13% switch and 55% on the right.
pub const BATS_GIVEN_THROWS_RIGHT: (f64, f64) = (0.24, 0.14);
pub const BATS_GIVEN_THROWS_LEFT: (f64, f64) = (0.88, 0.04);

/// Scorer's abbreviations for positions 1–9.
pub const POSITION_CODE: [&str; 9] = ["P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];

/// One side's nine players, indexed by position − 1.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamRoster {
    /// 0 away, 1 home.
    pub side: u8,
    pub name: &'static str,
    pub players: [Player; 9],
}

impl TeamRoster {
    /// The id a player bats and runs under (data-formats §2.4).
    pub fn bat_agent(&self, p: &Player) -> AgentId {
        ((self.side as u16) << 8) | p.batting_order as u16
    }

    /// The id he fields under.
    pub fn field_agent(&self, p: &Player) -> AgentId {
        ((self.side as u16) << 8) | 0x80 | p.position as u16
    }

    /// The players in batting order.
    pub fn batting_order(&self) -> [&Player; 9] {
        core::array::from_fn(|slot| self.players.iter().find(|p| p.batting_order as usize == slot + 1).expect("every batting slot is filled"))
    }
}

/// Both sides' rosters: index 0 away, 1 home.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rosters {
    pub teams: [TeamRoster; 2],
}

/// A draw of an index below `n` (data-formats §2.4): `floor(u · n)`, `u` in (0, 1).
fn index(rng: &mut KeyedRng, n: usize) -> usize {
    ((rng.uniform() * n as f64) as usize).min(n - 1)
}

fn stream(seed: u64, agent_id: AgentId, sub_index: u32) -> KeyedRng {
    KeyedRng::new(RngKey { game_seed: seed, play_index: 0, agent_id, purpose: Purpose::RosterGeneration, sub_index })
}

impl Rosters {
    /// The rosters of the game with this seed (data-formats §2.4): the same seed always gives
    /// the same players.
    pub fn generate(seed: u64) -> Rosters {
        let mut families: Vec<&'static str> = Vec::with_capacity(18);
        let teams = [0u8, 1].map(|side| {
            // The batting order: the manager shuffles the eight position players; the pitcher
            // bats ninth.
            let mut order: [u8; 8] = core::array::from_fn(|i| i as u8 + 2);
            let mut rng = stream(seed, ((side as u16) << 8) | 0xFF, 0);
            for i in (1..8).rev() {
                order.swap(i, index(&mut rng, i + 1));
            }
            let mut numbers: Vec<u8> = Vec::with_capacity(9);
            let mut givens: Vec<&'static str> = Vec::with_capacity(9);
            let players = core::array::from_fn(|i| {
                let position = i as u8 + 1;
                let agent = ((side as u16) << 8) | 0x80 | position as u16;
                let (origin, given, family) = (0u32..)
                    .find_map(|a| {
                        let mut rng = stream(seed, agent, 2 * a);
                        let u = rng.uniform();
                        let mut acc = 0.0;
                        let pool = NAME_POOLS.iter().find(|p| {
                            acc += p.weight;
                            u < acc
                        });
                        let pool = pool.unwrap_or(&NAME_POOLS[0]);
                        let given = pool.given[index(&mut rng, pool.given.len())];
                        let family = pool.family[index(&mut rng, pool.family.len())];
                        (!families.contains(&family) && !givens.contains(&given)).then_some((pool.origin, given, family))
                    })
                    .expect("the pools hold more family names than a game has players");
                families.push(family);
                givens.push(given);
                let number = (0u32..)
                    .find_map(|a| {
                        let n = 1 + index(&mut stream(seed, agent, 2 * a + 1), 99) as u8;
                        (!numbers.contains(&n)).then_some(n)
                    })
                    .expect("99 numbers for nine players");
                numbers.push(number);
                // Hands: the throwing hand from his position, then the batting side from it.
                let mut rng = KeyedRng::new(RngKey { game_seed: seed, play_index: 0, agent_id: agent, purpose: Purpose::RosterHands, sub_index: 0 });
                let throws = if rng.uniform() < THROWS_LEFT[i] { Hand::Left } else { Hand::Right };
                let (left, switch) = if throws == Hand::Left { BATS_GIVEN_THROWS_LEFT } else { BATS_GIVEN_THROWS_RIGHT };
                let u = rng.uniform();
                let bats = if u < left {
                    Bats::Left
                } else if u < left + switch {
                    Bats::Switch
                } else {
                    Bats::Right
                };
                let batting_order = if position == 1 { 9 } else { order.iter().position(|&p| p == position).expect("every position player bats") as u8 + 1 };
                Player { player_id: side as u32 * 9 + position as u32, given, family, origin, number, position, batting_order, bats, throws }
            });
            TeamRoster { side, name: if side == 0 { "Away" } else { "Home" }, players }
        });
        Rosters { teams }
    }

    /// The player behind an agent id, batting or fielding, and his team.
    pub fn find(&self, agent: AgentId) -> Option<(&TeamRoster, &Player)> {
        let team = self.teams.get((agent >> 8) as usize)?;
        let lo = agent & 0xFF;
        let p = if lo & 0x80 != 0 { team.players.iter().find(|p| p.position as u16 == lo & 0x7F) } else { team.players.iter().find(|p| p.batting_order as u16 == lo) };
        p.map(|p| (team, p))
    }

    /// The player behind an agent id, batting or fielding.
    pub fn player(&self, agent: AgentId) -> Option<&Player> {
        self.find(agent).map(|x| x.1)
    }

    /// The hand a side's pitcher throws with.
    pub fn pitcher_throws(&self, side: usize) -> Hand {
        self.teams[side].players[0].throws
    }

    /// Each side's batting ids in batting order, as the rules state takes them.
    pub fn lineups(&self) -> [[AgentId; 9]; 2] {
        [0, 1].map(|s| {
            let t = &self.teams[s];
            t.batting_order().map(|p| t.bat_agent(p))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// What data-formats §2.4 promises of any roster, checked over many seeds.
    #[test]
    fn every_roster_is_a_legal_pair_of_nines() {
        for seed in 0..500u64 {
            let r = Rosters::generate(seed);
            let mut ids = BTreeSet::new();
            let mut agents = BTreeSet::new();
            let mut families = BTreeSet::new();
            for (side, t) in r.teams.iter().enumerate() {
                assert_eq!(t.side as usize, side);
                // Nine positions, each filled once; nine batting slots, each filled once.
                assert_eq!(t.players.iter().map(|p| p.position).collect::<BTreeSet<_>>(), (1..=9).collect(), "seed {seed}");
                assert_eq!(t.players.iter().map(|p| p.batting_order).collect::<BTreeSet<_>>(), (1..=9).collect(), "seed {seed}");
                // Numbers 1–99, no two alike on a team.
                let numbers: BTreeSet<u8> = t.players.iter().map(|p| p.number).collect();
                assert_eq!(numbers.len(), 9, "seed {seed}: {numbers:?}");
                assert!(numbers.iter().all(|n| (1..=99).contains(n)));
                // No two teammates share a given name.
                assert_eq!(t.players.iter().map(|p| p.given).collect::<BTreeSet<_>>().len(), 9, "seed {seed}");
                // The pitcher bats ninth.
                assert_eq!(t.players.iter().find(|p| p.position == 1).unwrap().batting_order, 9);
                for p in &t.players {
                    assert!(ids.insert(p.player_id), "seed {seed}: player id {} twice", p.player_id);
                    assert!(agents.insert(t.bat_agent(p)) && agents.insert(t.field_agent(p)), "seed {seed}: an agent id twice");
                    assert!(families.insert(p.family), "seed {seed}: two players named {}", p.family);
                    assert!(!p.given.is_empty() && !p.family.is_empty());
                    // Either id finds the same man.
                    assert_eq!(r.player(t.bat_agent(p)), Some(p));
                    assert_eq!(r.player(t.field_agent(p)), Some(p));
                    // The name is a pairing from one pool.
                    let pool = NAME_POOLS.iter().find(|q| q.origin == p.origin).unwrap();
                    assert!(pool.given.contains(&p.given) && pool.family.contains(&p.family));
                }
            }
            assert_eq!((ids.len(), agents.len()), (18, 36));
            // The ids the code has always used: away batters 1–9, home 0x101–0x109.
            let fixture = bb_rules::RulesState::default_fixture().lineup;
            assert_eq!(r.lineups(), fixture);
        }
        // Not a player: an umpire, a manager, a batting slot that does not exist.
        let r = Rosters::generate(1);
        for id in [0x0200, 0x00FF, 0x0000, 0x000A, 0x0180, 0x018A, 0x0300] {
            assert_eq!(r.player(id), None, "{id:#x}");
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_players_and_another_seed_others() {
        assert_eq!(Rosters::generate(7), Rosters::generate(7));
        let names = |seed| Rosters::generate(seed).teams.iter().flat_map(|t| t.players.iter().map(|p| p.full_name())).collect::<Vec<_>>();
        assert_ne!(names(7), names(8));
        // The two sides of one game are different people.
        let r = Rosters::generate(7);
        assert_ne!(r.teams[0].players.iter().map(|p| p.family).collect::<Vec<_>>(), r.teams[1].players.iter().map(|p| p.family).collect::<Vec<_>>());
    }

    /// The draw contract of data-formats §2.4, pinned: seed 7's two nines in batting order
    /// (order, position, number, given, family). The Codex rosters review (2026-11-13)
    /// reproduced these players with its own Python Philox and §2.4 generator, so the
    /// literals are an independent expectation; a change to a stream id, a sub-index, the
    /// draw order, the shuffle or a name list breaks them. Seed 7 exercises the rejection of
    /// a teammate's given name (the away right fielder and the home second baseman were
    /// each drawn as a second "Yuki" and a second "Diego" first).
    #[test]
    fn seed_7_gives_these_players() {
        let want: [[(u8, &str, u8, &str, &str); 9]; 2] = [
            [
                (1, "CF", 28, "Yuki", "Kobayashi"),
                (2, "LF", 13, "Tyler", "Davis"),
                (3, "1B", 31, "Woo-jin", "Hwang"),
                (4, "SS", 22, "Julio", "Nunez"),
                (5, "RF", 68, "Justin", "Carter"),
                (6, "C", 70, "Austin", "Cooper"),
                (7, "2B", 94, "Kevin", "Roberts"),
                (8, "3B", 19, "Ryan", "Hayes"),
                (9, "P", 6, "Daniel", "Morgan"),
            ],
            [
                (1, "C", 66, "Logan", "Thompson"),
                (2, "2B", 65, "Raul", "Hernandez"),
                (3, "RF", 67, "Justin", "Phillips"),
                (4, "1B", 18, "Diego", "Cruz"),
                (5, "LF", 4, "Jordan", "Harris"),
                (6, "SS", 15, "Santiago", "Flores"),
                (7, "3B", 61, "Sam", "Baker"),
                (8, "CF", 19, "Koji", "Hayashi"),
                (9, "P", 82, "Haruto", "Fujita"),
            ],
        ];
        let r = Rosters::generate(7);
        for side in 0..2 {
            let got = r.teams[side].batting_order().map(|p| (p.batting_order, p.position_code(), p.number, p.given, p.family));
            assert_eq!(got, want[side], "side {side}");
        }
        // Hands, in batting order: batting side then throwing hand.
        let hands = ["S/R R/R S/R S/R L/L R/R L/R R/R S/R", "R/R S/R L/R S/R R/R L/R R/R R/R R/R"];
        for side in 0..2 {
            let got: Vec<String> = r.teams[side]
                .batting_order()
                .iter()
                .map(|p| {
                    let b = match p.bats {
                        Bats::Left => 'L',
                        Bats::Right => 'R',
                        Bats::Switch => 'S',
                    };
                    format!("{b}/{}", if p.throws == Hand::Left { 'L' } else { 'R' })
                })
                .collect();
            assert_eq!(got.join(" "), hands[side], "side {side}");
        }
        // Player ids: away 1–9 and home 10–18, in position order.
        assert_eq!(r.teams.iter().flat_map(|t| t.players.iter().map(|p| p.player_id)).collect::<Vec<_>>(), (1..=18).collect::<Vec<_>>());
    }

    /// Hands follow data-formats §2.4: nobody throws left at catcher, second, third or short;
    /// elsewhere the share is the position's; and the batting sides come out at the shares
    /// worked by hand from the two tables (position players: 13.125% throw left, so
    /// left 0.13125 × 0.88 + 0.86875 × 0.24 = 32.4%, switch 0.13125 × 0.04 + 0.86875 × 0.14 =
    /// 12.7%, right 54.9%).
    #[test]
    fn hands_follow_the_position_and_the_batting_side_follows_the_hand() {
        let n_seeds = 3000u64;
        let mut left = [0u32; 9];
        let mut bats = [0u32; 3];
        let (mut lefty_bats_right, mut lefties) = (0u32, 0u32);
        for seed in 0..n_seeds {
            for t in &Rosters::generate(seed).teams {
                for (i, p) in t.players.iter().enumerate() {
                    left[i] += (p.throws == Hand::Left) as u32;
                    if i > 0 {
                        bats[match p.bats {
                            Bats::Left => 0,
                            Bats::Switch => 1,
                            Bats::Right => 2,
                        }] += 1;
                        lefties += (p.throws == Hand::Left) as u32;
                        lefty_bats_right += (p.throws == Hand::Left && p.bats == Bats::Right) as u32;
                    }
                }
            }
        }
        let n = (2 * n_seeds) as f64;
        for (i, want) in [0.27, 0.0, 0.35, 0.0, 0.0, 0.0, 0.25, 0.20, 0.25].into_iter().enumerate() {
            let got = left[i] as f64 / n;
            // 6000 players a position: one sd is at most 0.6 points.
            assert!(if want == 0.0 { left[i] == 0 } else { (got - want).abs() < 0.025 }, "position {}: {got:.3} throw left, want {want}", i + 1);
        }
        let total = 8.0 * n;
        for (k, want) in [0.324, 0.127, 0.549].into_iter().enumerate() {
            let got = bats[k] as f64 / total;
            assert!((got - want).abs() < 0.01, "batting side {k}: {got:.3}, want {want}");
        }
        // A left-handed thrower who bats right is rare (8%), not absent.
        let share = lefty_bats_right as f64 / lefties as f64;
        assert!((share - 0.08).abs() < 0.02, "{share:.3}");
    }

    #[test]
    fn a_switch_hitter_bats_opposite_the_pitchers_hand() {
        assert_eq!(Bats::Switch.against(Hand::Right), Hand::Left);
        assert_eq!(Bats::Switch.against(Hand::Left), Hand::Right);
        for h in [Hand::Left, Hand::Right] {
            assert_eq!(Bats::Left.against(h), Hand::Left);
            assert_eq!(Bats::Right.against(h), Hand::Right);
        }
    }

    /// The pools as data-formats §2.4 describes them.
    #[test]
    fn name_pools_are_ascii_weighted_to_one_and_share_no_family_name() {
        assert!((NAME_POOLS.iter().map(|p| p.weight).sum::<f64>() - 1.0).abs() < 1e-12);
        let mut seen = BTreeSet::new();
        for pool in &NAME_POOLS {
            assert_eq!(pool.given.iter().collect::<BTreeSet<_>>().len(), pool.given.len(), "{:?}: a given name twice", pool.origin);
            for name in pool.given.iter().chain(pool.family) {
                assert!(name.bytes().all(|b| b.is_ascii_alphabetic() || b == b'-'), "{name}");
            }
            for f in pool.family {
                assert!(seen.insert(*f), "{f} is in two pools or twice in one");
            }
        }
        // More names than any game needs, so a rejected attempt always has acceptable names
        // left to draw (18 family names in a game, nine given names on a team). That makes
        // the loop end in practice; it is not a bound on the number of attempts.
        assert!(seen.len() > 18);
        assert!(NAME_POOLS.iter().map(|p| p.given.len()).sum::<usize>() > 9);
    }

    /// Over many rosters every pool's share is near its weight, every batting slot goes to
    /// every position player, and numbers spread over 1–99: the draws are not stuck.
    #[test]
    fn draws_cover_the_pools_the_order_and_the_numbers() {
        let n_seeds = 2000u64;
        let mut by_origin = [0u32; 5];
        let mut slot_pos = [[0u32; 10]; 10];
        let mut numbers = [0u32; 100];
        for seed in 0..n_seeds {
            for t in &Rosters::generate(seed).teams {
                for p in &t.players {
                    by_origin[p.origin as usize] += 1;
                    slot_pos[p.batting_order as usize][p.position as usize] += 1;
                    numbers[p.number as usize] += 1;
                }
            }
        }
        let total = (n_seeds * 18) as f64;
        for pool in &NAME_POOLS {
            // Rejections of a taken family name shift the shares a little; 2 points is far
            // more than sampling error at 36 000 players (0.26 points for the largest pool).
            let share = by_origin[pool.origin as usize] as f64 / total;
            assert!((share - pool.weight).abs() < 0.02, "{:?}: {share:.3} vs {}", pool.origin, pool.weight);
        }
        // Each of the 64 (slot, position) pairs expects 4000/8 = 500 teams; sd ≈ 21.
        for slot in 1..=8 {
            for pos in 2..=9 {
                let c = slot_pos[slot][pos];
                assert!((400..600).contains(&c), "slot {slot} position {pos}: {c}");
            }
        }
        // Numbers are uniform over 1–99: chi-squared against equal counts, 98 degrees of
        // freedom (its 0.1% point is 149).
        let expect = total / 99.0;
        let chi2: f64 = numbers[1..].iter().map(|&c| (c as f64 - expect) * (c as f64 - expect) / expect).sum();
        assert!(numbers[0] == 0 && chi2 < 149.0, "chi-squared {chi2:.1}");
    }
}
