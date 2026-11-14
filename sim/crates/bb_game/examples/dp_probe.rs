//! Double-play diagnostic (measuring only). `-- [games=256] [first_seed=0]`.
//!
//! A double-play chance is a fair ground ball (launch < 10°, the `sim-cli games` bins) with a
//! runner on first and fewer than two out. For each chance: who first secured the ball, how
//! many outs the play made, and for the plays that went fielder → pivot → first, the times
//! from contact: the fielder's glove-to-release (feed), the pivot's catch-to-release, the
//! ball's arrival at first and the batter's touch of first. Also plate-appearance-level
//! opportunities (Baseball-Reference's "GIDP Opp": runner on first, fewer than two out) and
//! how the line-drive and fly-ball plays with two or more outs were made.
use bb_game::game::Game;
use bb_rules::{Event, Input, OutKind};
use std::collections::BTreeMap;

const POS: [&str; 10] = ["?", "P", "C", "1B", "2B", "3B", "SS", "LF", "CF", "RF"];

#[derive(Default)]
struct Stats {
    n: u64,
    sum: f64,
    sq: f64,
}
impl Stats {
    fn add(&mut self, x: f64) {
        self.n += 1;
        self.sum += x;
        self.sq += x * x;
    }
    fn show(&self) -> String {
        if self.n == 0 {
            return "n 0".into();
        }
        let m = self.sum / self.n as f64;
        format!("n {} mean {:.3} sd {:.3}", self.n, m, (self.sq / self.n as f64 - m * m).max(0.0).sqrt())
    }
}

fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let games = a.first().copied().unwrap_or(256);
    let first = a.get(1).copied().unwrap_or(0);
    let (mut pa_opp, mut gidp, mut dp_all, mut gb_fair, mut chances) = (0u64, 0u64, 0u64, 0u64, 0u64);
    let mut by_fielder: BTreeMap<u16, (u64, u64)> = BTreeMap::new();
    let (mut feed, mut pivot, mut ball_first, mut batter_first, mut batter_first_all, mut contact_to_second) =
        (Stats::default(), Stats::default(), Stats::default(), Stats::default(), Stats::default(), Stats::default());
    let (mut attempts, mut attempts_dp) = (0u64, 0u64);
    // Statcast 2016's second-base set: his glove-to-release on plays where the batter was out
    // at first on his throw (grounders he fielded, and pivots).
    let (mut ex2_fielded, mut ex2_pivot) = (Stats::default(), Stats::default());
    let mut air_dp: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_type_dp = [0u64; 4];
    let mut fly_run = 0u64;
    for seed in first..first + games {
        let mut g = Game::new(seed, true);
        loop {
            let (outs0, on_first, any_on) = (g.st.outs, g.st.bases[0].is_some(), g.st.bases.iter().any(|b| b.is_some()));
            let Some(step) = g.step_pitch() else { break };
            let pa_end = step.rules_events.iter().any(|e| matches!(e, Event::PlateAppearanceEnded { .. }));
            if pa_end && on_first && outs0 < 2 {
                pa_opp += 1;
            }
            let Some((_, la, _)) = step.record.batted else { continue };
            let fair = step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true }));
            let caught = step.rules_events.iter().any(|e| matches!(e, Event::Catch { .. }));
            if !fair && !caught {
                continue;
            }
            let outs: Vec<(u16, OutKind, u8)> = step
                .rules_events
                .iter()
                .filter_map(|e| match *e {
                    Event::RunnerOut { runner, base, kind } => Some((runner, kind, base)),
                    _ => None,
                })
                .collect();
            let d = la.to_degrees();
            if caught && step.rules_events.iter().any(|e| matches!(e, Event::RunScored { .. })) && outs.iter().any(|o| o.1 == OutKind::FlyOut) {
                fly_run += 1;
            }
            let ty = if d < 10.0 { 0 } else if d < 25.0 { 1 } else if d < 50.0 { 2 } else { 3 };
            if std::env::var_os("DP_LIST").is_some() && caught && ty > 0 && any_on {
                eprintln!("caught {seed} {} {}", step.play, ["GB", "LD", "FB", "PU"][ty]);
            }
            if outs.len() >= 2 {
                dp_all += 1;
                by_type_dp[ty] += 1;
                if ty > 0 {
                    let key = format!("{} {:?}", ["GB", "LD", "FB", "PU"][ty], outs.iter().map(|o| (o.1, o.2)).collect::<Vec<_>>());
                    *air_dp.entry(key).or_default() += 1;
                    if std::env::var_os("DP_LIST").is_some() {
                        eprintln!("air2 {seed} {} {}", step.play, ["GB", "LD", "FB", "PU"][ty]);
                    }
                }
            }
            let tc = step.facts.iter().find_map(|f| if let Input::BatContact { t } = *f { Some(t) } else { None }).unwrap_or(0.0);
            let bat_first = step.facts.iter().find_map(|f| match *f {
                Input::BaseTouched { t, agent, base: 1, holding_ball: false } if agent == step.batter => Some(t),
                _ => None,
            });
            let batter_out_at_first = outs.iter().any(|o| o.0 == step.batter && o.2 == 1);
            if batter_out_at_first {
                let (s, r) = (&step.viz.secured, &step.viz.released);
                for k in 0..r.len().min(s.len().saturating_sub(1)) {
                    if s[k].1 & 0x7f == 4 && k + 1 == s.len() - 1 {
                        if k == 0 { ex2_fielded.add(r[k].0 - s[k].0) } else { ex2_pivot.add(r[k].0 - s[k].0) }
                    }
                }
            }
            if ty == 0 && fair {
                gb_fair += 1;
                if let Some(t) = bat_first {
                    batter_first_all.add(t - tc);
                }
            }
            if !(ty == 0 && fair && on_first && outs0 < 2) {
                continue;
            }
            chances += 1;
            let dp = outs.len() >= 2;
            gidp += dp as u64;
            let s = &step.viz.secured;
            let r = &step.viz.released;
            if let Some(&(_, f0)) = s.first() {
                let e = by_fielder.entry(f0 & 0x7f).or_default();
                e.0 += 1;
                e.1 += dp as u64;
            }
            // Fielder -> pivot man (who took it near second) -> first.
            if s.len() >= 3 && r.len() >= 2 {
                let near2 = step.viz.actors.iter().find(|(id, _)| *id == s[1].1).and_then(|(_, tr)| tr.iter().min_by(|x, y| (x.0 - s[1].0).abs().total_cmp(&(y.0 - s[1].0).abs())).map(|p| p.1));
                let second = bb_game::play::base_pos(2);
                if near2.is_some_and(|p| (p - second).norm() < 4.0) {
                    attempts += 1;
                    attempts_dp += dp as u64;
                    feed.add(r[0].0 - s[0].0);
                    pivot.add(r[1].0 - s[1].0);
                    contact_to_second.add(s[1].0 - tc);
                    ball_first.add(s[2].0 - tc);
                    if let Some(t) = bat_first {
                        batter_first.add(t - tc);
                    }
                }
            }
        }
    }
    let gm = games as f64;
    println!("{games} games, seeds {first}..{}", first + games - 1);
    println!("caught fly/line outs with a run scoring {fly_run} ({:.1} per 100 games)", 100.0 * fly_run as f64 / gm);
    println!("plays with 2+ outs {dp_all} ({:.2}/game); by type GB {} LD {} FB {} PU {}", dp_all as f64 / gm, by_type_dp[0], by_type_dp[1], by_type_dp[2], by_type_dp[3]);
    println!("PA opportunities (runner on 1st, <2 out) {pa_opp} ({:.1}/game); GB double plays in them {gidp} ({:.2}/game) = {:.1}% per opportunity", pa_opp as f64 / gm, gidp as f64 / gm, 100.0 * gidp as f64 / pa_opp as f64);
    println!("fair grounders {gb_fair}; DP-chance grounders {chances} ({:.1}/game, {:.1}% of grounders); converted {:.1}%", chances as f64 / gm, 100.0 * chances as f64 / gb_fair as f64, 100.0 * gidp as f64 / chances as f64);
    println!("by first fielder: share of chances, DP%");
    for (p, (n, k)) in &by_fielder {
        println!("  {:<3} {:5.1}% {:5.1}%", POS.get(*p as usize).unwrap_or(&"?"), 100.0 * *n as f64 / chances as f64, 100.0 * *k as f64 / *n as f64);
    }
    println!("fielder -> pivot at second -> first: {attempts} attempts, {attempts_dp} double plays");
    println!("  feed glove-to-release  {}", feed.show());
    println!("  contact to pivot catch {}", contact_to_second.show());
    println!("  pivot catch-to-release {}", pivot.show());
    println!("  contact to ball at 1st {}", ball_first.show());
    println!("  contact to batter at 1st (these plays) {}", batter_first.show());
    println!("2B exchange, batter out at first on his throw: fielded {}; pivots {}", ex2_fielded.show(), ex2_pivot.show());
    println!("batter contact to first, all fair grounders he reached it on: {}", batter_first_all.show());
    println!("LD/FB/PU plays with 2+ outs, by (kind, base) of the outs:");
    for (k, n) in &air_dp {
        println!("  {n:4} {k}");
    }
}
