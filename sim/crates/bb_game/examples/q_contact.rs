//! Read-only diagnostic: one CSV row per fair batted ball, as `ev_bat`, plus where on the
//! barrel the contact landed: `u_rel_in` (contact station minus the sweet spot, inches, +
//! toward the tip) and the off-centre angle between the contact normal and the bat point's
//! velocity (degrees) and the contact normal's vertical component `nz`.
//! Changes no game; prints a digest fold so a before/after check is possible.
//! `cargo run --release -p bb_game --example q_contact -- <seed-base> <n> <out.csv>`
use bb_rules::Event;
use std::io::Write;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let base: u64 = a[1].parse().unwrap();
    let n: u64 = a[2].parse().unwrap();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&a[3]).unwrap());
    writeln!(out, "game,batter,nominal_mph,bat_mph,pitch_mph,ev_mph,la_deg,spray_pull_deg,hr,u_rel_in,off_deg,nz").unwrap();
    let mu0 = bb_game::pa::Matchup::for_hands(bb_observe::Hand::Right, bb_observe::Hand::Right);
    let len = mu0.bat.profile.length();
    let u_ss = len - mu0.bat.contact.ss_from_tip;
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
            let (Some(sw), Some((tc, ball))) = (step.record.swing, step.record.contact_state) else { continue };
            use bb_physics::contact::BatPose;
            let k = sw.at(tc);
            let bat = k.point_velocity(k.point(u_ss), u_cm).norm();
            let u = (ball.r - k.knob).dot(k.axis).clamp(0.0, len);
            let p = k.point(u);
            let d = ball.r - p;
            let nrm = d * (1.0 / d.norm());
            let v = k.point_velocity(p, u_cm);
            let vperp = v - k.axis * v.dot(k.axis);
            let cosang = (nrm.dot(vperp) / (vperp.norm() * nrm.norm())).clamp(-1.0, 1.0);
            let lefty = sw.pivot.x > 0.0;
            let pull = if lefty { spray } else { -spray };
            let hr = step.rules_events.iter().any(|e| matches!(e, Event::PlateAppearanceEnded { result: bb_rules::PaResult::HomeRun, .. }));
            writeln!(
                out,
                "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{:.3},{:.3},{:.4}",
                i,
                step.batter,
                g.bat_speed((step.batter >> 8) as usize, (step.batter & 0xFF) as usize - 1) / 0.44704,
                bat / 0.44704,
                step.record.release.v.norm() / 0.44704,
                ev / 0.44704,
                la.to_degrees(),
                pull.to_degrees(),
                hr as u8,
                (u - u_ss) / 0.0254,
                cosang.acos().to_degrees(),
                nrm.z
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
