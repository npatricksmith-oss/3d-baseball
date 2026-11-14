//! Reproduce the B5 audit: cargo run --release -p bb_stats --example scorer_audit -- 300
use bb_schema::{ev, log};
use bb_stats::{box_score, scorer::{score_play, Outcome, DEFAULT_CHOICE_MARGIN_S}};

#[derive(Default)]
struct Counts { old_choices: u64, changed_to_hit: u64, converted_infield: u64, converted_outfield: u64, converted_unhandled: u64, old_hits_to_choice: u64, unresolved: u64, ab: u64, h: u64, hr: u64, k: u64, sf: u64, games: u64 }
impl Counts {
    fn add(&mut self, x: Self) {
        self.old_choices += x.old_choices; self.changed_to_hit += x.changed_to_hit;
        self.converted_infield += x.converted_infield; self.converted_outfield += x.converted_outfield; self.converted_unhandled += x.converted_unhandled;
        self.old_hits_to_choice += x.old_hits_to_choice; self.unresolved += x.unresolved;
        self.ab += x.ab; self.h += x.h; self.hr += x.hr; self.k += x.k; self.sf += x.sf; self.games += x.games;
    }
}
fn game(seed: u64) -> Counts {
    let g = bb_game::game::simulate_game(seed);
    let b = box_score(&g.log).expect("valid game log");
    let mut c = Counts { games: 1, ..Counts::default() };
    for (side, t) in b.teams.into_iter().enumerate() {
        let outs = b.outs_by_half.iter().skip(side).step_by(2).sum::<u32>();
        assert_eq!(t.pa, t.runs + t.left_on_base + outs, "I1 seed {seed}, side {side}");
        assert_eq!(t.pa, t.ab + t.bb + t.hbp + t.sf + t.sh, "I2 seed {seed}, side {side}");
        assert_eq!(t.h, t.singles + t.doubles + t.triples + t.hr, "I7 seed {seed}, side {side}");
        c.ab += t.ab as u64; c.h += t.h as u64; c.hr += t.hr as u64; c.k += t.k as u64; c.sf += t.sf as u64; c.unresolved += t.undetermined as u64;
    }
    let view = log::read(&g.log).unwrap();
    let mut old = g.bip_detail.iter();
    for p in &view.plays {
        let Some(events) = p.events() else { continue };
        let is_bip = events.iter().filter_map(|e| e.body_as_plate_appearance_ended()).any(|pa| pa.result() == ev::PaResult::Unknown);
        if !is_bip { continue; }
        let prior = old.next().expect("one diagnostic per BIP");
        let now = score_play(*p, 0, DEFAULT_CHOICE_MARGIN_S).unwrap().outcome;
        if prior.fielders_choice() {
            c.old_choices += 1;
            if matches!(now, Outcome::Hit(_)) {
                c.changed_to_hit += 1;
                let handling = events.iter().find_map(|e| e.body_as_scorer_evidence()).filter(|e| e.has_handling()).map(|e| e.first_handler() & 0x7f);
                match handling { Some(1..=6) => c.converted_infield += 1, Some(7..=9) => c.converted_outfield += 1, _ => c.converted_unhandled += 1 }
            }
        }
        if prior.hit && now == Outcome::FieldersChoice { c.old_hits_to_choice += 1; }
    }
    assert!(old.next().is_none());
    c
}
fn main() {
    let n: u64 = std::env::args().nth(1).unwrap_or_else(|| "300".into()).parse().unwrap();
    let counts = std::thread::scope(|s| {
        let handles: Vec<_> = (0..2).map(|worker| s.spawn(move || {
            let mut c = Counts::default();
            for seed in (worker..n).step_by(2) { c.add(game(seed)); }
            c
        })).collect();
        let mut c = Counts::default(); for h in handles { c.add(h.join().unwrap()); } c
    });
    println!("games {} old_FC {} became_hits {} old_hits_became_FC {} unresolved {}", counts.games, counts.old_choices, counts.changed_to_hit, counts.old_hits_to_choice, counts.unresolved);
    println!("converted first handler: infield {} outfield {} unknown {}", counts.converted_infield, counts.converted_outfield, counts.converted_unhandled);
    println!("AB {} H {} HR {} K {} SF {} AVG {:.6} BABIP {:.6}", counts.ab, counts.h, counts.hr, counts.k, counts.sf, counts.h as f64 / counts.ab as f64, (counts.h - counts.hr) as f64 / (counts.ab - counts.k - counts.hr + counts.sf) as f64);
}
