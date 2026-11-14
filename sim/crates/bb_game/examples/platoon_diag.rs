//! Platoon asymmetry diagnosis across R/R, L/L, R/L, L/R:
//! Measures pitch-type, location (inside/middle/outside), timing/vertical error, and runs
//! exact counterfactual interventions on perception, release, lateral break, and swing depth.
//!
//! Usage: `cargo run --release -p bb_game --example platoon_diag -- [pa_per_cell] [seed0]`

use bb_agents::batter::eye_point;
use bb_game::pa::{Call, Knobs, Matchup, PaResult, PA_PLAY_STRIDE, simulate_pa};
use bb_math::units::{FT, MPH};
use bb_observe::{Hand, PitchType};
use bb_physics::bat::{BatProfile, sweet_spot};
use bb_physics::contact::BatPose;
use bb_physics::ground::{Field, Segment, SegmentEnd, batted_path};
use bb_physics::zone::{HALF_WIDTH, Zone, ZoneRule};
use bb_world::pitch::pitch_flight;

fn is_barrel(ev_mph: f64, la_deg: f64) -> bool {
    if ev_mph < 98.0 {
        return false;
    }
    let e = ev_mph.floor();
    let (lo, hi) = if e >= 116.0 {
        (8.0, 50.0)
    } else {
        let d = e - 98.0;
        let lo = (26.0 - d).max(8.0);
        let hi_tbl = [
            30.0, 31.0, 33.0, 34.0, 35.0, 37.0, 38.0, 39.0, 40.0, 42.0, 43.0, 44.0, 45.0, 46.0,
            47.0, 48.0, 49.0, 50.0, 50.0,
        ];
        (lo, hi_tbl[(d as usize).min(18)])
    };
    la_deg >= lo && la_deg <= hi
}

fn pitch_idx(ty: PitchType) -> usize {
    match ty {
        PitchType::FourSeam => 0,
        PitchType::Slider => 1,
        PitchType::Changeup => 2,
        PitchType::Curveball => 3,
        _ => 0,
    }
}

#[derive(Default, Clone)]
struct CellStats {
    pa: u32,
    k: u32,
    called_k: u32,
    bb: u32,
    hbp: u32,
    bip: u32,
    of_hr: u32,
    pitches: u32,
    in_zone: u32,
    swings: u32,
    z_sw: u32,
    o_sw: u32,
    contact: u32,
    z_con: u32,
    o_con: u32,
    whiff: u32,
    foul: u32,
    cstr: u32,
    read_ok: u32,
    read_mat: [u32; 4],
    omega_early: Vec<f64>,
    omega_mid: Vec<f64>,
    omega_dec: Vec<f64>,
    sa_dec_mrad: Vec<f64>,
    all_contact_evs: Vec<f64>,
    evs: Vec<f64>,
    las: Vec<f64>,
    barrels: u32,
    bat_mphs: Vec<f64>,
    // error stats on swings (all swings / whiffs)
    aim_err_x: Vec<f64>,
    aim_err_z: Vec<f64>,
    aim_err_t: Vec<f64>,
    commit_err_x: Vec<f64>,
    commit_err_z: Vec<f64>,
    commit_err_t: Vec<f64>,
    whiff_ez_over6: u32,
    whiff_et_over25: u32,
    whiff_ex_over12: u32,
    whiff_kg_over2: u32,
}

impl CellStats {
    fn mean_sd(xs: &[f64]) -> (f64, f64) {
        if xs.is_empty() {
            return (0.0, 0.0);
        }
        let n = xs.len() as f64;
        let m = xs.iter().sum::<f64>() / n;
        let v = xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / n;
        (m, v.sqrt())
    }
    fn pct(a: u32, b: u32) -> f64 {
        100.0 * a as f64 / b.max(1) as f64
    }
}

fn eval_cell(
    mu: &Matchup,
    n_pa: u32,
    seed: u64,
    bats_hand: Hand,
    detailed: bool,
) -> (CellStats, [CellStats; 4], [CellStats; 3]) {
    let field = Field::default();
    let prof = BatProfile::default_34in();
    let mp = prof.mass_props();
    let uss = sweet_spot(prof.length());
    let _zone = Zone::for_batter(mu.batter.height, &ZoneRule::default());
    let eye = eye_point(bats_hand, mu.batter.height);
    let mut tot = CellStats::default();
    let mut by_ty: [CellStats; 4] = Default::default();
    let mut by_x: [CellStats; 3] = Default::default(); // 0=inside, 1=middle, 2=outside relative to batter

    for i in 0..n_pa {
        let pa = simulate_pa(mu, seed, i * PA_PLAY_STRIDE);
        tot.pa += 1;
        match pa.result {
            PaResult::Strikeout => {
                tot.k += 1;
                if pa.called_third {
                    tot.called_k += 1;
                }
            }
            PaResult::Walk => tot.bb += 1,
            PaResult::HitByPitch => tot.hbp += 1,
            PaResult::InPlay => tot.bip += 1,
            PaResult::Unresolved => {}
        }
        for r in &pa.pitches {
            let pi = pitch_idx(r.ty);
            let x_rel = r.plate.map(|p| {
                let s = match bats_hand {
                    Hand::Right => 1.0, // RHB: -X is inside, +X is outside
                    Hand::Left => -1.0, // LHB: +X is inside, -X is outside
                };
                p.x * s
            }).unwrap_or(0.0);
            let xi = if x_rel < -HALF_WIDTH * 0.45 {
                0 // inside
            } else if x_rel <= HALF_WIDTH * 0.45 {
                1 // middle
            } else {
                2 // outside
            };
            let (om_e, om_m, om_d, sa_d) = if detailed {
                let pf = pitch_flight(&mu.env, r.release);
                let om_at = |t: f64| {
                    let (p, v) = pf.at(t.min(pf.last().t));
                    let dv = p - eye;
                    dv.cross(v).norm() / dv.norm_sq().max(1e-6)
                };
                let od = om_at(r.t_decide);
                let sa = mu.perception.angular_sigma(od) * mu.knobs.perception * (1.0 - mu.perception.persistent_share).sqrt() * 1000.0;
                (Some(om_at(0.05)), Some(om_at(0.20)), Some(od), Some(sa))
            } else {
                (None, None, None, None)
            };

            let update = |cs: &mut CellStats| {
                cs.pitches += 1;
                if let (Some(oe), Some(om), Some(od), Some(sa)) = (om_e, om_m, om_d, sa_d) {
                    cs.omega_early.push(oe);
                    cs.omega_mid.push(om);
                    cs.omega_dec.push(od);
                    cs.sa_dec_mrad.push(sa);
                }
                if r.in_zone {
                    cs.in_zone += 1;
                }
                if r.read == r.ty {
                    cs.read_ok += 1;
                }
                cs.read_mat[pitch_idx(r.read)] += 1;
                if r.contact {
                    if let Some((ev, _, _)) = r.batted {
                        cs.all_contact_evs.push(ev / MPH);
                    }
                }
                if r.swung {
                    cs.swings += 1;
                    if r.in_zone {
                        cs.z_sw += 1;
                    } else {
                        cs.o_sw += 1;
                    }
                    if r.contact {
                        cs.contact += 1;
                        if r.in_zone {
                            cs.z_con += 1;
                        } else {
                            cs.o_con += 1;
                        }
                        if matches!(r.call, Call::Foul | Call::FoulTip) {
                            cs.foul += 1;
                        }
                    } else {
                        cs.whiff += 1;
                        if let Some(ae) = r.aim_error {
                            if ae.y.abs() > 0.06 {
                                cs.whiff_ez_over6 += 1;
                            }
                            if ae.z.abs() > 0.025 {
                                cs.whiff_et_over25 += 1;
                            }
                            if ae.x.abs() > 0.12 {
                                cs.whiff_ex_over12 += 1;
                            }
                        }
                        if let Some(kg) = r.known_gap {
                            if kg.y.abs() > 0.02 {
                                cs.whiff_kg_over2 += 1;
                            }
                        }
                    }
                    if detailed {
                        if let Some(ae) = r.aim_error {
                            cs.aim_err_x.push(ae.x * 100.0);
                            cs.aim_err_z.push(ae.y * 100.0);
                            cs.aim_err_t.push(ae.z * 1000.0);
                        }
                        if let Some(ce) = r.commit_error {
                            cs.commit_err_x.push(ce.x * 100.0);
                            cs.commit_err_z.push(ce.y * 100.0);
                            cs.commit_err_t.push(ce.z * 1000.0);
                        }
                    }
                } else if matches!(r.call, Call::CalledStrike) {
                    cs.cstr += 1;
                }
                if matches!(r.call, Call::InPlay) {
                    if let (Some((ev, la, _)), Some((tc, s0))) = (r.batted, r.contact_state) {
                        let ev_mph = ev / MPH;
                        let la_deg = la.to_degrees();
                        cs.evs.push(ev_mph);
                        cs.las.push(la_deg);
                        if is_barrel(ev_mph, la_deg) {
                            cs.barrels += 1;
                        }
                        if let Some(sw) = r.swing {
                            let k = BatPose::at(&sw, tc);
                            cs.bat_mphs.push(k.point_velocity(k.point(uss), mp.u_cm).norm() / MPH);
                        }
                        if detailed {
                            let path = batted_path(&mu.env, &field, s0, tc, |_, _| (0.0, 0.0));
                            let of = path.segments.iter().any(|s| {
                                matches!(s, Segment::Flight { end: SegmentEnd::OverFence { fair: true }, .. })
                            });
                            if of {
                                cs.of_hr += 1;
                            }
                        }
                    }
                }
            };
            update(&mut tot);
            if detailed {
                update(&mut by_ty[pi]);
                update(&mut by_x[xi]);
            }
        }
    }
    let _ = FT;
    (tot, by_ty, by_x)
}

fn make_mu(throws: Hand, bats: Hand) -> Matchup {
    let mut mu = Matchup::for_hands(throws, bats);
    mu.knobs = Knobs::calibrated_m3();
    mu
}

fn main() {
    let n_pa: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    let seed: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(3);
    let pairs = [
        ("R/R", Hand::Right, Hand::Right),
        ("L/L", Hand::Left, Hand::Left),
        ("R/L", Hand::Right, Hand::Left),
        ("L/R", Hand::Left, Hand::Right),
    ];

    println!("=== DETAILED 4-CELL PLATOON DIAGNOSTIC ({n_pa} PA/cell, seed {seed}, M3 knobs) ===");
    let mut cells = Vec::new();
    for &(lbl, th, bh) in &pairs {
        let mu = make_mu(th, bh);
        let (tot, by_ty, by_x) = eval_cell(&mu, n_pa, seed, bh, true);
        let (ev_m, _) = CellStats::mean_sd(&tot.evs);
        let (la_m, la_sd) = CellStats::mean_sd(&tot.las);
        let hh = 100.0 * tot.evs.iter().filter(|&&e| e >= 95.0).count() as f64 / tot.evs.len().max(1) as f64;
        let brl = CellStats::pct(tot.barrels, tot.evs.len() as u32);
        let mut bs = tot.bat_mphs.clone();
        bs.sort_by(|a, b| a.total_cmp(b));
        let bat_p90 = if bs.is_empty() { 0.0 } else { bs[(0.9 * (bs.len() - 1) as f64) as usize] };
        println!(
            "\n[{lbl}] K% {:.1} (CK {:.1}%) BB% {:.1} BIP% {:.1} OF-HR/PA {:.2}% (HR/BIP {:.1}%) | Z% {:.1} Sw% {:.1} ZSw% {:.1} OSw% {:.1} | Con% {:.1} ZCon% {:.1} OCon% {:.1} SwStr% {:.1} Foul/C% {:.1} Read% {:.1}",
            CellStats::pct(tot.k, tot.pa),
            CellStats::pct(tot.called_k, tot.k),
            CellStats::pct(tot.bb, tot.pa),
            CellStats::pct(tot.bip, tot.pa),
            CellStats::pct(tot.of_hr, tot.pa),
            CellStats::pct(tot.of_hr, tot.bip),
            CellStats::pct(tot.in_zone, tot.pitches),
            CellStats::pct(tot.swings, tot.pitches),
            CellStats::pct(tot.z_sw, tot.in_zone),
            CellStats::pct(tot.o_sw, tot.pitches - tot.in_zone),
            CellStats::pct(tot.contact, tot.swings),
            CellStats::pct(tot.z_con, tot.z_sw),
            CellStats::pct(tot.o_con, tot.o_sw),
            CellStats::pct(tot.whiff, tot.pitches),
            CellStats::pct(tot.foul, tot.contact),
            CellStats::pct(tot.read_ok, tot.pitches),
        );
        println!(
            "  BIP: EV {ev_m:.1} (AllContactEV {:.1}) HH% {hh:.1} Brl% {brl:.1} LA {la_m:.1}±{la_sd:.1} BatSpd {:.1} (p90 {bat_p90:.1}) | omega(0.05/0.20/dec) {:.2}/{:.2}/{:.2} rad/s sa_dec {:.2} mrad",
            CellStats::mean_sd(&tot.all_contact_evs).0,
            CellStats::mean_sd(&tot.bat_mphs).0,
            CellStats::mean_sd(&tot.omega_early).0,
            CellStats::mean_sd(&tot.omega_mid).0,
            CellStats::mean_sd(&tot.omega_dec).0,
            CellStats::mean_sd(&tot.sa_dec_mrad).0,
        );
        let (ax_m, ax_s) = CellStats::mean_sd(&tot.aim_err_x);
        let (az_m, az_s) = CellStats::mean_sd(&tot.aim_err_z);
        let (at_m, at_s) = CellStats::mean_sd(&tot.aim_err_t);
        let (cx_m, cx_s) = CellStats::mean_sd(&tot.commit_err_x);
        let (cz_m, cz_s) = CellStats::mean_sd(&tot.commit_err_z);
        let (ct_m, ct_s) = CellStats::mean_sd(&tot.commit_err_t);
        println!(
            "  Errors: Commit(x,z cm; t ms) ({cx_m:+.1}±{cx_s:.1}, {cz_m:+.1}±{cz_s:.1}, {ct_m:+.1}±{ct_s:.1}) | FinalAim ({ax_m:+.1}±{ax_s:.1}, {az_m:+.1}±{az_s:.1}, {at_m:+.1}±{at_s:.1})"
        );
        println!(
            "  Whiffs (n={}): |ez|>6cm {:.1}%, |et|>25ms {:.1}%, |ex|>12cm {:.1}%, known_gap_z>2cm {:.1}%",
            tot.whiff,
            CellStats::pct(tot.whiff_ez_over6, tot.whiff),
            CellStats::pct(tot.whiff_et_over25, tot.whiff),
            CellStats::pct(tot.whiff_ex_over12, tot.whiff),
            CellStats::pct(tot.whiff_kg_over2, tot.whiff),
        );
        println!("  By PitchType:");
        for (pi, pn) in ["FF", "SL", "CH", "CU"].iter().enumerate() {
            let c = &by_ty[pi];
            println!(
                "    {pn}: P {:4} Z% {:4.1} OSw% {:4.1} ZCon% {:4.1} OCon% {:4.1} SwStr% {:4.1} Foul/C% {:4.1} Read% {:4.1} read->[{:2.0},{:2.0},{:2.0},{:2.0}] omega_d {:.2} sa {:.2}mrad | EV {:4.1} HR/BIP {:4.1}% | Whiff ez>6 {:4.1}% et>25 {:4.1}%",
                c.pitches,
                CellStats::pct(c.in_zone, c.pitches),
                CellStats::pct(c.o_sw, c.pitches - c.in_zone),
                CellStats::pct(c.z_con, c.z_sw),
                CellStats::pct(c.o_con, c.o_sw),
                CellStats::pct(c.whiff, c.pitches),
                CellStats::pct(c.foul, c.contact),
                CellStats::pct(c.read_ok, c.pitches),
                CellStats::pct(c.read_mat[0], c.pitches),
                CellStats::pct(c.read_mat[1], c.pitches),
                CellStats::pct(c.read_mat[2], c.pitches),
                CellStats::pct(c.read_mat[3], c.pitches),
                CellStats::mean_sd(&c.omega_dec).0,
                CellStats::mean_sd(&c.sa_dec_mrad).0,
                CellStats::mean_sd(&c.evs).0,
                CellStats::pct(c.of_hr, c.evs.len() as u32),
                CellStats::pct(c.whiff_ez_over6, c.whiff),
                CellStats::pct(c.whiff_et_over25, c.whiff),
            );
        }
        println!("  By Horizontal Plate Half (batter frame):");
        for (xi, xn) in ["Inside", "Middle", "Outside"].iter().enumerate() {
            let c = &by_x[xi];
            let (_, azs) = CellStats::mean_sd(&c.aim_err_z);
            let (_, ats) = CellStats::mean_sd(&c.aim_err_t);
            let (_, axs) = CellStats::mean_sd(&c.aim_err_x);
            println!(
                "    {xn:7}: P {:4} Sw% {:4.1} ZCon% {:4.1} OCon% {:4.1} SwStr% {:4.1} Read% {:4.1} EV {:4.1} Bat {:4.1} HR/BIP {:4.1}% | AimSD(x,z,t) ({axs:.1}cm, {azs:.1}cm, {ats:.1}ms)",
                c.pitches,
                CellStats::pct(c.swings, c.pitches),
                CellStats::pct(c.z_con, c.z_sw),
                CellStats::pct(c.o_con, c.o_sw),
                CellStats::pct(c.whiff, c.pitches),
                CellStats::pct(c.read_ok, c.pitches),
                CellStats::mean_sd(&c.evs).0,
                CellStats::mean_sd(&c.bat_mphs).0,
                CellStats::pct(c.of_hr, c.evs.len() as u32),
            );
        }
        cells.push(tot);
    }

    // ---- Causal Mechanism Intervention Matrix (2000 PA/cell)
    let int_n = n_pa.min(2000);
    println!("\n=== MECHANISM INTERVENTION TABLE ({int_n} PA/cell, seed {seed}) ===");
    println!(
        "{:36} | {:>5} {:>5} {:>5} {:>5} | {:>6} {:>6} | {:>6} {:>6} | {:>6}",
        "Intervention", "R/R K", "L/L K", "R/L K", "L/R K", "Gap K", "GapSwStr", "SameZCon", "OppZCon", "SameOCon/Opp"
    );

    let run_int = |label: &str, f: &dyn Fn(&mut Matchup)| {
        let mut ks = [0.0; 4];
        let mut sws = [0.0; 4];
        let mut zcs = [0.0; 4];
        let mut ocs = [0.0; 4];
        for (i, &(_, th, bh)) in pairs.iter().enumerate() {
            let mut mu = make_mu(th, bh);
            f(&mut mu);
            mu.refresh();
            let (c, _, _) = eval_cell(&mu, int_n, seed, bh, false);
            ks[i] = CellStats::pct(c.k, c.pa);
            sws[i] = CellStats::pct(c.whiff, c.pitches);
            zcs[i] = CellStats::pct(c.z_con, c.z_sw);
            ocs[i] = CellStats::pct(c.o_con, c.o_sw);
        }
        let same_k = 0.5 * (ks[0] + ks[1]);
        let opp_k = 0.5 * (ks[2] + ks[3]);
        let same_sw = 0.5 * (sws[0] + sws[1]);
        let opp_sw = 0.5 * (sws[2] + sws[3]);
        let same_zc = 0.5 * (zcs[0] + zcs[1]);
        let opp_zc = 0.5 * (zcs[2] + zcs[3]);
        let same_oc = 0.5 * (ocs[0] + ocs[1]);
        let opp_oc = 0.5 * (ocs[2] + ocs[3]);
        println!(
            "{label:36} | {:5.1} {:5.1} {:5.1} {:5.1} | {:+6.2} {:+8.2} | {:6.1}% {:6.1}% | {:4.1}% / {:4.1}%",
            ks[0], ks[1], ks[2], ks[3], same_k - opp_k, same_sw - opp_sw, same_zc, opp_zc, same_oc, opp_oc
        );
    };

    run_int("0. Baseline (M3)", &|_| {});
    run_int("1. Pursuit off (limit=1e6)", &|m| m.perception.pursuit_limit = 1e6);
    run_int("2. No sideways break (tilt 12/6)", &|m| {
        for a in &mut m.arsenal {
            a.def.tilt_rhp = if (a.def.tilt_rhp - 6.0).abs() < 3.0 { 6.0 } else { 12.0 };
        }
    });
    run_int("3. Centered release (arm_slot x=0)", &|m| {
        m.pitcher.arm_slot = (-0.25f64).acos(); // x_rel == 0 exactly
    });
    run_int("4. Centered rel + No side break", &|m| {
        m.pitcher.arm_slot = (-0.25f64).acos();
        for a in &mut m.arsenal {
            a.def.tilt_rhp = if (a.def.tilt_rhp - 6.0).abs() < 3.0 { 6.0 } else { 12.0 };
        }
    });
    run_int("5. Pursuit off + No side break", &|m| {
        m.perception.pursuit_limit = 1e6;
        for a in &mut m.arsenal {
            a.def.tilt_rhp = if (a.def.tilt_rhp - 6.0).abs() < 3.0 { 6.0 } else { 12.0 };
        }
    });
    run_int("6. Zero Weber (tau noise off)", &|m| m.perception.weber = 1e-4);
    run_int("7. Zero angular noise (sigma_ang=1e-5)", &|m| m.perception.sigma_ang = 1e-5);
    run_int("8. Zero spin noise (sigma_s0=0)", &|m| m.perception.sigma_s0 = 0.0);
    run_int("9. Exact scouting (model_error=None)", &|m| m.model_error = None);
    run_int("10. Flat contact_y (contact_k=0)", &|m| m.template.contact_k = 0.0);
    run_int("11. Zero swing noise (k_swing=0)", &|m| m.knobs.swing = 0.0);
    run_int("12. k_perception = 1.0", &|m| m.knobs.perception = 1.0);
    run_int("13. Outside r_knob_max = 0.28", &|m| m.template.r_knob_max = 0.28);
    run_int("14. Discipline b = 1.4 (more chase)", &|m| m.discipline.b = 1.4);

    // Extra detailed print of R/R vs R/L under Pursuit Off + Clamp r_knob_max + Discipline b=1.4
    println!("\n=== R/R vs R/L DETAIL UNDER PURSUIT OFF (limit=1e6, 2000 PA) ===");
    for &(lbl, th, bh) in &[("R/R", Hand::Right, Hand::Right), ("R/L", Hand::Right, Hand::Left)] {
        let mut mu = make_mu(th, bh);
        mu.perception.pursuit_limit = 1e6;
        mu.refresh();
        let (c, bt, bx) = eval_cell(&mu, int_n, seed, bh, true);
        println!("[{lbl} pursuit_off] K% {:.1} (CK {:.1}%) BB% {:.1} OSw% {:.1} ZSw% {:.1} ZCon% {:.1} OCon% {:.1} SwStr% {:.1} Foul/C% {:.1} Read% {:.1}",
            CellStats::pct(c.k, c.pa), CellStats::pct(c.called_k, c.k), CellStats::pct(c.bb, c.pa),
            CellStats::pct(c.o_sw, c.pitches - c.in_zone), CellStats::pct(c.z_sw, c.in_zone),
            CellStats::pct(c.z_con, c.z_sw), CellStats::pct(c.o_con, c.o_sw),
            CellStats::pct(c.whiff, c.pitches), CellStats::pct(c.foul, c.contact), CellStats::pct(c.read_ok, c.pitches));
        for (pi, pn) in ["FF", "SL", "CH", "CU"].iter().enumerate() {
            let t = &bt[pi];
            println!("  {pn}: ZCon% {:.1} OCon% {:.1} OSw% {:.1} SwStr% {:.1} Read% {:.1}",
                CellStats::pct(t.z_con, t.z_sw), CellStats::pct(t.o_con, t.o_sw),
                CellStats::pct(t.o_sw, t.pitches - t.in_zone), CellStats::pct(t.whiff, t.pitches), CellStats::pct(t.read_ok, t.pitches));
        }
        for (xi, xn) in ["Inside", "Middle", "Outside"].iter().enumerate() {
            let x = &bx[xi];
            println!("  {xn:7}: ZCon% {:.1} OCon% {:.1} SwStr% {:.1} Read% {:.1} AimSD(x,z,t) ({:.1}cm,{:.1}cm,{:.1}ms)",
                CellStats::pct(x.z_con, x.z_sw), CellStats::pct(x.o_con, x.o_sw),
                CellStats::pct(x.whiff, x.pitches), CellStats::pct(x.read_ok, x.pitches),
                CellStats::mean_sd(&x.aim_err_x).1, CellStats::mean_sd(&x.aim_err_z).1, CellStats::mean_sd(&x.aim_err_t).1);
        }
    }

    println!("\n=== R/R LEVER-ARM & DISCIPLINE DIAGNOSTIC (2000 PA) ===");
    for &(lbl, rmax, disc_b, bat_v) in &[
        ("Baseline (r_max=0.70, b=0.9, v=32)", 0.70, 0.9, 32.0),
        ("r_knob_max = 0.28 (cap out bat spd)", 0.28, 0.9, 32.0),
        ("Discipline b = 1.4 (OSw toward 31%)", 0.70, 1.4, 32.0),
        ("Bat speed 30.5 rad/s (~2-str/power)", 0.70, 0.9, 30.5),
    ] {
        let mut mu = make_mu(Hand::Right, Hand::Right);
        mu.template.r_knob_max = rmax;
        mu.discipline.b = disc_b;
        mu.batter.bat_speed = bat_v;
        mu.refresh();
        let (c, _, _) = eval_cell(&mu, int_n, seed, Hand::Right, true);
        let hh = 100.0 * c.evs.iter().filter(|&&e| e >= 95.0).count() as f64 / c.evs.len().max(1) as f64;
        let p90_bat = {
            let mut v = c.bat_mphs.clone();
            v.sort_by(|a, b| a.total_cmp(b));
            if v.is_empty() { 0.0 } else { v[(0.9 * (v.len() - 1) as f64) as usize] }
        };
        println!("  {lbl:37} | K% {:4.1} BB% {:4.1} OF-HR/PA {:4.2}% | Z% {:4.1} OSw% {:4.1} Con% {:4.1} | EV {:4.1} HH% {:4.1} Brl% {:4.1} Bat(p50/p90) {:.1}/{:.1}",
            CellStats::pct(c.k, c.pa), CellStats::pct(c.bb, c.pa), CellStats::pct(c.of_hr, c.pa),
            CellStats::pct(c.in_zone, c.pitches), CellStats::pct(c.o_sw, c.pitches - c.in_zone), CellStats::pct(c.contact, c.swings),
            CellStats::mean_sd(&c.evs).0, hh, CellStats::pct(c.barrels, c.evs.len() as u32),
            CellStats::mean_sd(&c.bat_mphs).0, p90_bat);
    }

    println!("\n=== POST-DECIDE OMEGA(t) & ESTIMATE-ERROR TRACE (2000 PA, R/R vs R/L) ===");
    for &(lbl, th, bh, p_lim) in &[
        ("R/R pursuit ON", Hand::Right, Hand::Right, 1.2),
        ("R/R pursuit OFF", Hand::Right, Hand::Right, 1e6),
        ("R/L pursuit ON", Hand::Right, Hand::Left, 1.2),
        ("R/L pursuit OFF", Hand::Right, Hand::Left, 1e6),
    ] {
        let mut mu = make_mu(th, bh);
        mu.perception.pursuit_limit = p_lim;
        mu.refresh();
        let eye = eye_point(bh, mu.batter.height);
        let (mut sw_n, mut over_pre, mut over_cut, mut om_pre_sum, mut om_cut_sum, mut y_cut_sum) = (0u32, 0u32, 0u32, 0.0, 0.0, 0.0);
        let (mut est_z, mut est_t, mut kg_z, mut kg_t) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut by_ty_cut = [(0u32, 0u32, 0.0f64); 4]; // (sw, >1.2 at cut, sum om_cut)
        let mut by_x_cut: [(u32, u32, f64, Vec<f64>, Vec<f64>); 3] = core::array::from_fn(|_| (0, 0, 0.0, Vec::new(), Vec::new()));
        for i in 0..int_n {
            let pa = simulate_pa(&mu, seed, i * PA_PLAY_STRIDE);
            for r in &pa.pitches {
                let Some(sw) = r.swing else { continue };
                sw_n += 1;
                let pf = pitch_flight(&mu.env, r.release);
                let om_at = |t: f64| {
                    let (p, v) = pf.at(t.clamp(0.0, pf.last().t));
                    let dv = p - eye;
                    (dv.cross(v).norm() / dv.norm_sq().max(1e-6), p.y)
                };
                let (om_d, _) = om_at(r.t_decide);
                let t_cut = (sw.t_c - mu.template.adj_cutoff_s).max(r.t_decide);
                let (om_c, y_c) = om_at(t_cut);
                if om_d > 1.2 { over_pre += 1; }
                if om_c > 1.2 { over_cut += 1; }
                om_pre_sum += om_d;
                om_cut_sum += om_c;
                y_cut_sum += y_c;
                let pi = pitch_idx(r.ty);
                by_ty_cut[pi].0 += 1;
                if om_c > 1.2 { by_ty_cut[pi].1 += 1; }
                by_ty_cut[pi].2 += om_c;
                let s = if bh == Hand::Right { 1.0 } else { -1.0 };
                let x_rel = r.plate.map(|p| p.x * s).unwrap_or(0.0);
                let xi = if x_rel < -HALF_WIDTH * 0.45 { 0 } else if x_rel <= HALF_WIDTH * 0.45 { 1 } else { 2 };
                by_x_cut[xi].0 += 1;
                if om_c > 1.2 { by_x_cut[xi].1 += 1; }
                by_x_cut[xi].2 += om_c;
                if let Some(ee) = r.est_error {
                    est_z.push(ee.y * 100.0);
                    est_t.push(ee.z * 1000.0);
                    by_x_cut[xi].3.push(ee.y * 100.0);
                }
                if let Some(ae) = r.aim_error {
                    by_x_cut[xi].4.push(ae.y * 100.0);
                }
                if let Some(kg) = r.known_gap {
                    kg_z.push(kg.y * 100.0);
                    kg_t.push(kg.z * 1000.0);
                }
            }
        }
        println!("  [{lbl}] swings={sw_n} | om_dec={:.2} (>1.2: {:.1}%) -> om_cut={:.2} (>1.2: {:.1}%) at y_cut={:.2}m | EstSD(z={:.1}cm, t={:.1}ms) | KnownGapSD(z={:.1}cm, t={:.1}ms)",
            om_pre_sum / sw_n.max(1) as f64, 100.0 * over_pre as f64 / sw_n.max(1) as f64,
            om_cut_sum / sw_n.max(1) as f64, 100.0 * over_cut as f64 / sw_n.max(1) as f64,
            y_cut_sum / sw_n.max(1) as f64,
            CellStats::mean_sd(&est_z).1, CellStats::mean_sd(&est_t).1,
            CellStats::mean_sd(&kg_z).1, CellStats::mean_sd(&kg_t).1);
        for (pi, pn) in ["FF", "SL", "CH", "CU"].iter().enumerate() {
            let (n, ov, sm) = by_ty_cut[pi];
            println!("    {pn}: om_cut={:.2} (>1.2: {:.1}%)", sm / n.max(1) as f64, 100.0 * ov as f64 / n.max(1) as f64);
        }
        for (xi, xn) in ["Inside", "Middle", "Outside"].iter().enumerate() {
            let (n, ov, sm, ref ez, ref az) = by_x_cut[xi];
            println!("    {xn:7}: om_cut={:.2} (>1.2: {:.1}%) EstZ_SD={:.1}cm AimZ_SD={:.1}cm",
                sm / n.max(1) as f64, 100.0 * ov as f64 / n.max(1) as f64,
                CellStats::mean_sd(ez).1, CellStats::mean_sd(az).1);
        }
    }
}
