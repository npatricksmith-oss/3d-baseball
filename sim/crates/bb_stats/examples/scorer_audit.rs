//! Reproduce the B5 audit (docs/calibration/2026-11-14-scorer.md):
//!   cargo run --release -p bb_stats --example scorer_audit -- 300 [first_seed]
//! Plays every game, compares the older lab classifier (`classify_bip`, which reads facts the
//! log does not carry and makes no scorer judgments) with the log-backed scorer on every ball
//! in play, checks the team accounting identities on each game's box score, and prints the
//! batting average and BABIP by batted-ball type with sample captions of each kind of ruling.
use bb_game::describe::describe_scored;
use bb_game::game::{Game, classify_bip};
use bb_rules::Event;
use bb_stats::{box_score, scorer::Outcome};

const TYPES: [&str; 4] = ["GB (<10°)", "LD (10–25°)", "FB (25–50°)", "PU (>50°)"];
fn bin(launch_deg: f64) -> usize {
    if launch_deg < 10.0 { 0 } else if launch_deg < 25.0 { 1 } else if launch_deg < 50.0 { 2 } else { 3 }
}

#[derive(Default)]
struct Counts {
    games: u64,
    old_choices: u64,
    old_choice_now_hit: u64,
    old_choice_now_choice: u64,
    old_hit_now_choice: u64,
    old_hit_now_throw_award: u64,
    unresolved: u64,
    thrown_awards: u64,
    inside_park_hr: u64,
    ab: u64, h: u64, hr: u64, k: u64, sf: u64, bb: u64, hbp: u64,
    // Per batted-ball type, balls in play only (home runs are not in play here, as in BABIP).
    bip: [u64; 4], old_hits: [u64; 4], new_hits: [u64; 4], new_choices: [u64; 4], new_outs: [u64; 4], award: [u64; 4],
    samples: Vec<String>,
}
impl Counts {
    fn add(&mut self, x: Self) {
        self.games += x.games; self.old_choices += x.old_choices; self.old_choice_now_hit += x.old_choice_now_hit;
        self.old_choice_now_choice += x.old_choice_now_choice; self.old_hit_now_choice += x.old_hit_now_choice;
        self.old_hit_now_throw_award += x.old_hit_now_throw_award;
        self.unresolved += x.unresolved; self.thrown_awards += x.thrown_awards; self.inside_park_hr += x.inside_park_hr;
        self.ab += x.ab; self.h += x.h; self.hr += x.hr; self.k += x.k; self.sf += x.sf; self.bb += x.bb; self.hbp += x.hbp;
        for i in 0..4 {
            self.bip[i] += x.bip[i]; self.old_hits[i] += x.old_hits[i]; self.new_hits[i] += x.new_hits[i];
            self.new_choices[i] += x.new_choices[i]; self.new_outs[i] += x.new_outs[i]; self.award[i] += x.award[i];
        }
        // Three captions of each kind are enough to read.
        for line in x.samples {
            let kind = line.split(':').next().unwrap_or("").to_string();
            if self.samples.iter().filter(|s| s.starts_with(&kind)).count() < 3 { self.samples.push(line); }
        }
    }
}

fn game(seed: u64) -> Counts {
    let mut g = Game::new(seed, true);
    let mut c = Counts { games: 1, ..Counts::default() };
    while let Some(p) = g.step_pitch() {
        let Some(batter) = p.rules_events.iter().find_map(|e| if let Event::PlateAppearanceEnded { batter, result: bb_rules::PaResult::BattedBall } = e { Some(*batter) } else { None }) else { continue };
        let ruling = p.scorer_ruling.expect("recorded game carries the scorer's ruling");
        // An inside-the-park home run is a home run: not a ball in play for BABIP (review P2-2).
        if ruling.outcome == Outcome::Hit(4) {
            c.inside_park_hr += 1;
            continue;
        }
        let (_, old_hit) = classify_bip(batter, &p.rules_events, &p.facts);
        let old_choice = classify_bip(batter, &p.rules_events, &p.facts).0 && !old_hit;
        let t = bin(p.record.batted.map(|b| b.1.to_degrees()).unwrap_or(0.0));
        c.bip[t] += 1;
        c.old_hits[t] += old_hit as u64;
        let tag = |c: &mut Counts, what: &str| {
            if c.samples.iter().filter(|s| s.starts_with(what)).count() < 3 {
                c.samples.push(format!("{what}: seed {seed} play {}: {}", p.play, describe_scored(p.record.call, p.record.batted.map(|b| b.1.to_degrees()), &p.rules_events, &p.facts, None, Some(ruling))));
            }
        };
        match ruling.outcome {
            Outcome::Hit(_) => c.new_hits[t] += 1,
            Outcome::FieldersChoice => c.new_choices[t] += 1,
            Outcome::ReachedOnThrowAward => { c.award[t] += 1; c.thrown_awards += 1; }
            Outcome::Undetermined => { c.unresolved += 1; tag(&mut c, "unresolved"); }
            _ => c.new_outs[t] += 1,
        }
        if old_choice {
            c.old_choices += 1;
            match ruling.outcome {
                Outcome::Hit(_) => { c.old_choice_now_hit += 1; tag(&mut c, "was FC, now hit"); }
                Outcome::FieldersChoice => { c.old_choice_now_choice += 1; tag(&mut c, "still FC"); }
                _ => {}
            }
        }
        if old_hit {
            match ruling.outcome {
                Outcome::FieldersChoice => { c.old_hit_now_choice += 1; tag(&mut c, "was hit, now FC"); }
                Outcome::ReachedOnThrowAward => { c.old_hit_now_throw_award += 1; tag(&mut c, "was hit, now throw award"); }
                _ => {}
            }
        }
    }
    let result = g.finish();
    let b = box_score(&result.log).expect("valid game log");
    for (side, t) in b.teams.iter().enumerate() {
        let outs = b.outs_by_half.iter().skip(side).step_by(2).sum::<u32>();
        assert_eq!(t.pa, t.runs + t.left_on_base + outs, "I1 seed {seed}, side {side}");
        assert_eq!(t.pa, t.ab + t.bb + t.hbp + t.sf + t.sh, "I2 seed {seed}, side {side}");
        assert_eq!(t.h, t.singles + t.doubles + t.triples + t.hr, "I7 seed {seed}, side {side}");
        c.ab += t.ab as u64; c.h += t.h as u64; c.hr += t.hr as u64; c.k += t.k as u64; c.sf += t.sf as u64; c.bb += t.bb as u64; c.hbp += t.hbp as u64;
    }
    c
}

fn main() {
    let n: u64 = std::env::args().nth(1).unwrap_or_else(|| "300".into()).parse().unwrap();
    let first: u64 = std::env::args().nth(2).map(|a| a.parse().unwrap()).unwrap_or(0);
    let counts = std::thread::scope(|s| {
        let handles: Vec<_> = (0..2).map(|worker| s.spawn(move || {
            let mut c = Counts::default();
            for seed in (first + worker..first + n).step_by(2) { c.add(game(seed)); }
            c
        })).collect();
        let mut c = Counts::default();
        for h in handles { c.add(h.join().unwrap()); }
        c
    });
    let c = &counts;
    println!("games {} (seeds {first}..{})", c.games, first + n);
    println!("old classifier called {} plays fielder's choices: now hits {}, still choices {}, other {}", c.old_choices, c.old_choice_now_hit, c.old_choice_now_choice, c.old_choices - c.old_choice_now_hit - c.old_choice_now_choice);
    println!("old classifier hits now ruled fielder's choice {}, now bases-awarded-on-a-throw {}", c.old_hit_now_choice, c.old_hit_now_throw_award);
    println!("thrown-ball awards {}; unresolved rulings {}; inside-the-park home runs (not in play) {}", c.thrown_awards, c.unresolved, c.inside_park_hr);
    println!("\n{:<13}{:>7}{:>10}{:>10}{:>8}{:>8}{:>9}{:>10}", "type", "BIP", "old hits", "new hits", "FC", "award", "old BABIP", "new BABIP");
    for i in 0..4 {
        // The old classifier counted hits per ball in play; the scorer's hits per ball in
        // play are the same denominator, so the two columns compare directly.
        println!("{:<13}{:>7}{:>10}{:>10}{:>8}{:>8}{:>9.3}{:>10.3}", TYPES[i], c.bip[i], c.old_hits[i], c.new_hits[i], c.new_choices[i], c.award[i],
            c.old_hits[i] as f64 / c.bip[i] as f64, c.new_hits[i] as f64 / c.bip[i] as f64);
    }
    let tot = |a: &[u64; 4]| a.iter().sum::<u64>();
    println!("{:<13}{:>7}{:>10}{:>10}{:>8}{:>8}{:>9.3}{:>10.3}", "all in play", tot(&c.bip), tot(&c.old_hits), tot(&c.new_hits), tot(&c.new_choices), tot(&c.award),
        tot(&c.old_hits) as f64 / tot(&c.bip) as f64, tot(&c.new_hits) as f64 / tot(&c.bip) as f64);
    println!("\nAB {} H {} HR {} K {} BB {} HBP {} SF {}", c.ab, c.h, c.hr, c.k, c.bb, c.hbp, c.sf);
    println!("AVG {:.4}  BABIP (H−HR)/(AB−K−HR+SF) {:.4}", c.h as f64 / c.ab as f64, (c.h - c.hr) as f64 / (c.ab - c.k - c.hr + c.sf) as f64);
    println!("\nsample captions:");
    let mut samples = counts.samples;
    samples.sort();
    for s in samples { println!("  {s}"); }
}
