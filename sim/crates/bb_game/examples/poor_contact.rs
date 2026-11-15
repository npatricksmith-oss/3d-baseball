//! Read-only diagnostic for the poor-contact work (docs/calibration/2026-11-15-collision-q-and-poor-contact.md):
//! one CSV row per bat-ball contact in games, fair or foul, with `q_contact`'s columns plus
//! the contact depth and lateral intercept (ball position at contact, inches), the contact
//! instant relative to the executed swing's nominal contact (ms, + = the ball met after it),
//! and the swing's effort (sweet-spot speed at its nominal instant over the batter's BatSpeed,
//! minus 1), and the off-centre offset across the bat (mm, + = ball above the bat point's
//! path; Higuchi et al. 2016's impact Z deviation). Changes no game; prints a digest fold for before/after checks.
//! `cargo run --release -p bb_game --example poor_contact -- <seed-base> <n> <out.csv>`
use bb_rules::Event;
use std::io::Write;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let base: u64 = a[1].parse().unwrap();
    let n: u64 = a[2].parse().unwrap();
    let mut out = std::io::BufWriter::new(std::fs::File::create(&a[3]).unwrap());
    writeln!(out, "game,batter,hand,fair,nominal_mph,bat_mph,pitch_mph,ev_mph,la_deg,hr,u_rel_in,off_deg,depth_in,lat_in,t_err_ms,effort,voff_mm").unwrap();
    let mu0 = bb_game::pa::Matchup::for_hands(bb_observe::Hand::Right, bb_observe::Hand::Right);
    let len = mu0.bat.profile.length();
    let u_ss = len - mu0.bat.contact.ss_from_tip;
    let u_cm = mu0.bat.mass.u_cm;
    let mut fold = [0u8; 32];
    for i in 0..n {
        let seed = base * 1_000_003 + i;
        let mut g = bb_game::game::Game::new(seed, false);
        while let Some(step) = g.step_pitch() {
            let (Some(sw), Some((tc, ball))) = (step.record.swing, step.record.contact_state) else { continue };
            let Some((ev, la, _)) = step.record.batted else { continue };
            let fair = step.rules_events.iter().any(|e| matches!(e, Event::FairFoulDecided { fair: true }));
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
            // Off-centre (vertical) offset: the ball centre's distance from the bat point's line
            // of motion, across the bat, + when the ball is above it (the bat under the ball).
            // Higuchi et al. 2016's "impact Z deviation" in the bat frame.
            let side = k.axis.cross(vperp);
            let side = side * (1.0 / side.norm());
            let side = if side.z < 0.0 { side * -1.0 } else { side };
            let voff = d.dot(side);
            let lefty = sw.pivot.x > 0.0;
            let nominal = g.bat_speed((step.batter >> 8) as usize, (step.batter & 0xFF) as usize - 1);
            let k0 = sw.at(sw.t_c);
            let effort = k0.point_velocity(k0.point(u_ss), u_cm).norm() / nominal - 1.0;
            let hr = step.rules_events.iter().any(|e| matches!(e, Event::PlateAppearanceEnded { result: bb_rules::PaResult::HomeRun, .. }));
            writeln!(
                out,
                "{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.5},{:.2}",
                i,
                step.batter,
                if lefty { "L" } else { "R" },
                fair as u8,
                nominal / 0.44704,
                bat / 0.44704,
                step.record.release.v.norm() / 0.44704,
                ev / 0.44704,
                la.to_degrees(),
                hr as u8,
                (u - u_ss) / 0.0254,
                cosang.acos().to_degrees(),
                ball.r.y / 0.0254,
                (ball.r.x - sw.pivot.x).abs() / 0.0254,
                (tc - sw.t_c) * 1000.0,
                effort,
                voff * 1000.0,
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
