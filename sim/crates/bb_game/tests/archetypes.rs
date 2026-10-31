//! M3 gate (PLAN §14): archetypes produce distinct profiles, in the physical directions.

use bb_agents::attributes::{PitcherGrades, archetypes};
use bb_game::pa::{B1Report, Matchup, run_pas};

fn ev(r: &B1Report) -> f64 {
    r.bbe_ev.iter().sum::<f64>() / r.bbe_ev.len().max(1) as f64
}
fn k(r: &B1Report) -> f64 {
    r.k as f64 / r.pa as f64
}
fn contact(r: &B1Report) -> f64 {
    r.contacts as f64 / r.swings.max(1) as f64
}
fn chase(r: &B1Report) -> f64 {
    r.o_swings as f64 / (r.pitches - r.zone).max(1) as f64
}

#[test]
fn batter_archetypes_are_distinct() {
    let bs = archetypes::batters();
    let reps: Vec<(&str, B1Report)> = std::thread::scope(|s| {
        let hs: Vec<_> = bs.iter().map(|(n, g)| s.spawn(move || (*n, run_pas(&Matchup::with_grades(g, &PitcherGrades::default()), 1000, 31)))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let get = |name: &str| &reps.iter().find(|(n, _)| *n == name).unwrap().1;
    let (avg, power, cont, free, pat, elite, weak) =
        (get("Average"), get("Power"), get("Contact"), get("Free swinger"), get("Patient"), get("Elite"), get("Weak"));
    assert!(ev(power) > ev(avg) + 2.0 && ev(power) > ev(cont) + 2.0);
    assert!(k(cont) < k(avg) && contact(cont) > contact(power));
    assert!(chase(free) > chase(avg) && chase(pat) < chase(avg));
    assert!(k(elite) < k(avg) && ev(elite) > ev(avg) && k(weak) > k(avg) && ev(weak) < ev(avg));
}
