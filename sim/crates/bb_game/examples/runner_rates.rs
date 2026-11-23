//! Diagnostic for the runner decision model (docs/calibration/2026-11-15-runner-decision-model.md
//! §4, T1, T2, T4, T6): how often a runner takes the extra base on a hit and how often he is
//! put out going, by the outs before the play; tag-ups from third on caught outfield flies;
//! and outs on the bases by cause, per team-game. `-- [games] [first seed]`;
//! `BB_SEND_TWO_OUT_Z=<z>` runs with another caution for the two-out send home.
use bb_game::{game::{classify_bip, Game}, play::FielderAttrs};
use bb_rules::{Event, Input, OutKind};
use bb_stats::scorer::{score_play, Outcome, DEFAULT_CHOICE_MARGIN_S};

#[derive(Default, Clone, Copy)]
struct Counts {
    // [situation][outs] = [opportunities, took the base, put out on the play]
    c: [[[usize; 3]; 3]; 3],
    // Runner on third, fewer than two out, fly caught in the outfield: [chances, scored, put out].
    tag: [[usize; 3]; 2],
    // Outs of runners other than force outs and the batter's own: on a hit (batter, other),
    // after a catch (doubled off or thrown out tagging), other plays.
    oob: [usize; 4],
    team_games: usize,
}

fn run(seeds: std::ops::Range<u64>) -> Counts {
    let mut k = Counts::default();
    let (c, tag) = (&mut k.c, &mut k.tag);
    let (mut oob_hit_batter, mut oob_hit_other, mut oob_catch, mut oob_other, mut team_games) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for seed in seeds {
        let mut g = Game::new(seed, false).with_fielders(core::array::from_fn(FielderAttrs::average));
        if let Some(z) = std::env::var("BB_SEND_TWO_OUT_Z").ok().and_then(|z| z.parse().ok()) {
            g = g.with_advance(bb_agents::running::AdvanceParams { send_two_out_z: z, ..Default::default() });
        }
        let mut data = Vec::new();
        loop {
            let (bases, outs_before) = (g.st.bases, g.st.outs);
            let Some(s) = g.step_pitch() else { break };
            let (reached, hit) = classify_bip(s.batter, &s.rules_events, &s.facts);
            let caught = s.rules_events.iter().any(|e| matches!(e, Event::RunnerOut { kind: OutKind::FlyOut, .. }));
            let mut outs_here = Vec::new();
            for e in &s.rules_events {
                if let Event::RunnerOut { runner, kind, .. } = e {
                    if matches!(kind, OutKind::Tag | OutKind::Appeal) || (caught && *kind == OutKind::Force) {
                        outs_here.push(*runner);
                        if caught {
                            oob_catch += 1
                        } else if reached && hit {
                            if *runner == s.batter { oob_hit_batter += 1 } else { oob_hit_other += 1 }
                        } else {
                            oob_other += 1
                        }
                    }
                }
            }
            if caught && outs_before < 2 {
                if let Some(id) = bases[2] {
                    let deep = s.facts.iter().find_map(|f| if let Input::GloveSecured { x, y, .. } = f { Some(x * x + y * y > 55.0 * 55.0) } else { None }).unwrap_or(false);
                    if deep {
                        let t = &mut tag[outs_before as usize];
                        t[0] += 1;
                        if s.rules_events.iter().any(|e| matches!(e, Event::RunScored { runner, .. } if *runner == id)) { t[1] += 1 }
                        if outs_here.contains(&id) { t[2] += 1 }
                    }
                }
            }
            data.push((s.play, bases, outs_before, s.facts, outs_here));
        }
        let res = g.finish();
        team_games += 2;
        let log = bb_schema::log::read(&res.log).unwrap();
        for (ix, bases, ob, facts, outs_here) in data {
            let Some(sc) = score_play(log.plays[ix as usize], ob as u32, DEFAULT_CHOICE_MARGIN_S) else { continue };
            let Outcome::Hit(hb) = sc.outcome else { continue };
            for (i, start, end, hitbase) in [(0, 1, 3, 1), (1, 2, 4, 1), (2, 1, 4, 2)] {
                if hb != hitbase { continue }
                if let Some(id) = bases[start - 1] {
                    let x = &mut c[i][ob as usize];
                    x[0] += 1;
                    if facts.iter().any(|f| matches!(f, Input::BaseTouched { agent, base, holding_ball: false, .. } if *agent == id && *base >= end)) { x[1] += 1 }
                    if outs_here.contains(&id) { x[2] += 1 }
                }
            }
        }
    }
    k.oob = [oob_hit_batter, oob_hit_other, oob_catch, oob_other];
    k.team_games = team_games;
    k
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let n: u64 = a.get(1).map(|x| x.parse().unwrap()).unwrap_or(100);
    let s0: u64 = a.get(2).map(|x| x.parse().unwrap()).unwrap_or(0);
    const THREADS: u64 = 6;
    let parts: Vec<Counts> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..THREADS).map(|i| sc.spawn(move || run(s0 + n * i / THREADS..s0 + n * (i + 1) / THREADS))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let (mut c, mut tag, mut oob, mut team_games) = ([[[0usize; 3]; 3]; 3], [[0usize; 3]; 2], [0usize; 4], 0usize);
    for p in parts {
        for i in 0..3 { for o in 0..3 { for j in 0..3 { c[i][o][j] += p.c[i][o][j] } } }
        for o in 0..2 { for j in 0..3 { tag[o][j] += p.tag[o][j] } }
        for j in 0..4 { oob[j] += p.oob[j] }
        team_games += p.team_games;
    }
    let [oob_hit_batter, oob_hit_other, oob_catch, oob_other] = oob;
    let pct = |a: usize, b: usize| if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 };
    println!("games {n} from seed {s0}");
    for (i, (name, real)) in [("1st to 3rd on a single", "31.7 / 1.1"), ("2nd to home on a single", "61.7 / 3.3"), ("1st to home on a double", "38.1 / 2.5")].iter().enumerate() {
        let tot: [usize; 3] = core::array::from_fn(|j| (0..3).map(|o| c[i][o][j]).sum());
        print!("{name}: n {} took {:.1}% out {:.1}% (MLB {real}) | by outs", tot[0], pct(tot[1], tot[0]), pct(tot[2], tot[0]));
        for o in 0..3 { print!("  {o}: n {} took {:.0}% out {:.1}%", c[i][o][0], pct(c[i][o][1], c[i][o][0]), pct(c[i][o][2], c[i][o][0])) }
        println!();
    }
    for o in 0..2 { println!("tag from third, outfield fly caught, {o} out: n {} scored {:.1}% out {:.1}% (MLB all: 75-80 / 2-3.5)", tag[o][0], pct(tag[o][1], tag[o][0]), pct(tag[o][2], tag[o][0])) }
    let tg = team_games as f64;
    println!("outs on the bases per team-game: on a hit, batter {:.3}, other runners {:.3}; after a catch {:.3}; other plays {:.3}; total {:.3} (MLB all causes 0.268; the three hit advances 0.065)", oob_hit_batter as f64 / tg, oob_hit_other as f64 / tg, oob_catch as f64 / tg, oob_other as f64 / tg, (oob_hit_batter + oob_hit_other + oob_catch + oob_other) as f64 / tg);
}
