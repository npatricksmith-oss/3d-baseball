//! One line per plate appearance of a seed's game, from the event log alone, for scoring by hand:
//!   cargo run --release -p bb_stats --example dump_plays -- <seed>
use bb_schema::ev::{self, EventBody};
use bb_schema::log;
fn main() {
    let seed: u64 = std::env::args().nth(1).unwrap().parse().unwrap();
    let g = bb_game::game::simulate_game(seed);
    let b = bb_stats::box_score(&g.log).unwrap();
    let name = |id: u16| -> String {
        for s in 0..2 { if let Some(l) = b.batting[s].iter().find(|l| l.agent == id) { return format!("{}{}", l.family, l.batting_order); } }
        format!("#{id}")
    };
    let view = log::read(&g.log).unwrap();
    let (mut inn, mut half) = (0, "");
    for p in &view.plays {
        let Some(events) = p.events() else { continue };
        let mut line = String::new();
        let mut pa = None;
        for e in events.iter() {
            match e.body_type() {
                EventBody::HalfInningStarted => { let h = e.body_as_half_inning_started().unwrap(); inn = h.inning(); half = if h.half() == ev::HalfInning::Top { "T" } else { "B" }; }
                EventBody::PlateAppearanceEnded => pa = e.body_as_plate_appearance_ended(),
                EventBody::RunnerOut => { let o = e.body_as_runner_out().unwrap(); line += &format!(" OUT {} {:?}@{}", name(o.runner()), o.kind(), o.base()); }
                EventBody::RunnerAdvanced => { let a = e.body_as_runner_advanced().unwrap(); line += &format!(" ADV {} {}->{} {:?}", name(a.runner()), a.from_base(), a.to_base(), a.cause()); }
                EventBody::RunScored => line += &format!(" RUN {}", name(e.body_as_run_scored().unwrap().runner())),
                EventBody::BallDead => line += &format!(" DEAD {:?}", e.body_as_ball_dead().unwrap().reason()),
                EventBody::GameEnded => line += &format!(" GAME_END {:?}", e.body_as_game_ended().unwrap().reason()),
                EventBody::ScorerEvidence => {
                    let s = e.body_as_scorer_evidence().unwrap();
                    let pos = |x: u16| x & 0x7f;
                    let rs: Vec<String> = s.runners().unwrap().iter().filter(|r| r.base() > 0).map(|r| format!("{}@{}", name(r.agent()), r.base())).collect();
                    let thr: Vec<String> = s.throws().map(|ts| ts.iter().map(|t| format!("thr>{}{}{} t{:.1}", t.base(), if t.attempted_out() { "!" } else { "" }, if t.runner() > 0 { format!("({})", name(t.runner())) } else { String::new() }, t.t_release())).collect()).unwrap_or_default();
                    let car: Vec<String> = s.carries().map(|cs| cs.iter().map(|c| format!("carry>{}({})", c.base(), name(c.runner()))).collect()).unwrap_or_default();
                    let at = s.secured_at().map(|p| (p.x() * p.x() + p.y() * p.y()).sqrt()).unwrap_or(0.0);
                    line += &format!(" [on {:?} fld{} {:.0}m 1st{} margin{} eta{} {:?} {:?}]", rs, if s.has_secured() { pos(s.reference_fielder()) } else { 0 }, at,
                        if s.has_first_touch() { format!("{:.1}", s.first_touch_s()) } else { "-".into() },
                        if s.has_margin() { format!("{:+.2}", s.clean_play_margin_s()) } else { "-".into() },
                        if s.has_batter_eta() { format!("{:.1}", s.batter_eta_s()) } else { "-".into() }, thr, car);
                }
                _ => {}
            }
        }
        if let Some(pa) = pa { println!("{inn}{half} {} {:?}{line}", name(pa.batter()), pa.result()); }
    }
    println!("FINAL {:?} linescore {:?}", g.runs, g.linescore);
}
