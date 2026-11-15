//! Read-only diagnostic: one CSV row per fair batted ball with the batter's identity, his
//! drawn BatSpeed, the sweet-spot speed at the executed contact instant, the release speed, and
//! the batted-ball facts. For the power-mix note (who supplies the 105+ mph balls).
//! Changes no game; prints a digest fold so a before/after check is possible.
//! `cargo run --release -p bb_game --example ev_bat -- <seed-base> <n> <out.csv>`
//! (game seeds are `base * 1_000_003 + i`, as in `sim-cli games`).
use bb_rules::Event;
use std::io::Write;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let base: u64 = a[1].parse().unwrap();
    let n: u64 = a[2].parse().unwrap();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&a[3]).unwrap());
    writeln!(out, "game,batter,nominal_mph,bat_mph,pitch_mph,ev_mph,la_deg,spray_pull_deg,hr").unwrap();
    let mu0 = bb_game::pa::Matchup::for_hands(bb_observe::Hand::Right, bb_observe::Hand::Right);
    let u_ss = mu0.bat.profile.length() - mu0.bat.contact.ss_from_tip;
    let u_cm = mu0.bat.mass.u_cm;
    let mut fold = [0u8; 32];
    for i in 0..n {
        let seed = base * 1_000_003 + i;
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let Some((ev, la, spray)) = step.record.batted else { continue };
            if !step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true })) {
                continue;
            }
            let (Some(sw), Some((tc, _))) = (step.record.swing, step.record.contact_state) else { continue };
            use bb_physics::contact::BatPose;
            let k = sw.at(tc);
            let bat = k.point_velocity(k.point(u_ss), u_cm).norm();
            let lefty = sw.pivot.x > 0.0;
            let pull = if lefty { spray } else { -spray };
            let hr = step.rules_events.iter().any(|e| matches!(e, Event::PlateAppearanceEnded { result: bb_rules::PaResult::HomeRun, .. }));
            writeln!(
                out,
                "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{}",
                i,
                step.batter,
                g.bat_speed((step.batter >> 8) as usize, (step.batter & 0xFF) as usize - 1) / 0.44704,
                bat / 0.44704,
                step.record.release.v.norm() / 0.44704,
                ev / 0.44704,
                la.to_degrees(),
                pull.to_degrees(),
                hr as u8
            )
            .unwrap();
        }
        let d = g.finish().digest;
        for (f, b) in fold.iter_mut().zip(d.iter()) {
            *f ^= b;
        }
    }
    let hex: String = fold.iter().map(|b| format!("{b:02x}")).collect();
    println!("games {n} fold {hex}");
}
