//! `sim-cli sensitivity` (PLAN §13.2, calibration-targets §7): sweep one attribute at a time,
//! with common random numbers, and check that each moves its primary metric the right way.

use bb_agents::attributes::{BatterGrades, PitcherGrades};
use bb_game::pa::{B1Report, Matchup, run_pas};
use clap::Args;

#[derive(Args)]
pub struct SensArgs {
    #[arg(long, default_value_t = 1500)]
    n: u32,
    #[arg(long, default_value_t = 3)]
    seed: u64,
}

type Setter = fn(&mut BatterGrades, &mut PitcherGrades, f64);
type Metric = fn(&B1Report) -> f64;

struct Case {
    attr: &'static str,
    set: Setter,
    metric_name: &'static str,
    metric: Metric,
    /// +1: metric should rise with the grade; −1: fall.
    sign: f64,
}

fn rate(a: u32, b: u32) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}
fn mean_ev(r: &B1Report) -> f64 {
    r.bbe_ev.iter().sum::<f64>() / r.bbe_ev.len().max(1) as f64
}

fn cases() -> Vec<Case> {
    vec![
        Case { attr: "BatSpeed", set: |b, _, g| b.bat_speed = g, metric_name: "mean EV", metric: mean_ev, sign: 1.0 },
        Case { attr: "BatControl", set: |b, _, g| b.bat_control = g, metric_name: "Contact%", metric: |r| rate(r.contacts, r.swings), sign: 1.0 },
        Case { attr: "Timing", set: |b, _, g| b.timing = g, metric_name: "Contact%", metric: |r| rate(r.contacts, r.swings), sign: 1.0 },
        Case { attr: "Vision", set: |b, _, g| b.vision = g, metric_name: "Contact%", metric: |r| rate(r.contacts, r.swings), sign: 1.0 },
        Case { attr: "SpinRecognition", set: |b, _, g| b.spin_recognition = g, metric_name: "SwStr%", metric: |r| rate(r.whiffs, r.pitches), sign: -1.0 },
        Case { attr: "Discipline", set: |b, _, g| b.discipline = g, metric_name: "O-Swing%", metric: |r| rate(r.o_swings, r.pitches - r.zone), sign: -1.0 },
        Case { attr: "QuickHands", set: |b, _, g| b.quick_hands = g, metric_name: "Contact%", metric: |r| rate(r.contacts, r.swings), sign: 1.0 },
        Case { attr: "Command (P)", set: |_, p, g| p.command = g, metric_name: "BB%", metric: |r| rate(r.bb, r.pa), sign: -1.0 },
        Case { attr: "ArmSpeed (P)", set: |_, p, g| p.arm_speed = g, metric_name: "Contact%", metric: |r| rate(r.contacts, r.swings), sign: -1.0 },
        Case { attr: "SpinCapacity (P)", set: |_, p, g| p.spin_capacity = g, metric_name: "SwStr%", metric: |r| rate(r.whiffs, r.pitches), sign: 1.0 },
    ]
}

pub fn sensitivity(a: SensArgs) {
    let grades = [30.0, 50.0, 70.0];
    let cs = cases();
    let results: Vec<Vec<f64>> = std::thread::scope(|sc| {
        let hs: Vec<_> = cs
            .iter()
            .map(|c| {
                let (n, seed) = (a.n, a.seed);
                sc.spawn(move || {
                    grades
                        .iter()
                        .map(|&g| {
                            let (mut bg, mut pg) = (BatterGrades::default(), PitcherGrades::default());
                            (c.set)(&mut bg, &mut pg, g);
                            let mu = Matchup::with_grades(&bg, &pg);
                            (c.metric)(&run_pas(&mu, n, seed))
                        })
                        .collect()
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    println!("{} PA per point, seed {} (common random numbers), interim M3 knobs", a.n, a.seed);
    println!("{:18} {:10} {:>8} {:>8} {:>8}  {:>10}  check", "attribute", "metric", "g30", "g50", "g70", "Δ per 10g");
    let mut fails = 0;
    for (c, r) in cs.iter().zip(&results) {
        let slope = (r[2] - r[0]) / 4.0;
        let ok = c.sign * (r[2] - r[0]) > 0.0;
        fails += !ok as u32;
        println!("{:18} {:10} {:8.2} {:8.2} {:8.2}  {:+10.2}  {}", c.attr, c.metric_name, r[0], r[1], r[2], slope, if ok { "ok" } else { "WRONG DIRECTION" });
    }
    println!("{} of {} in the expected direction", cs.len() as u32 - fails, cs.len());
}
