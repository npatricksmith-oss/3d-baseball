//! Where the bat's knob is, from the batter's standing spot, through his swings: what the
//! client's coded swing (docs/player-visuals.md, §9 and §13) has to put the hands on. For each
//! swing, mirrored so every batter is right-handed (he stands at x < 0, the plate toward +x,
//! the pitcher toward +y): the knob at the swing's start, at nominal contact and at the end
//! of the bat track (nominal contact + 0.12 s), the hand radius, the height of the hand-arc's
//! centre and its distance from the standing spot, and the times.
//!
//! §13 build step 1 adds the measurements the base swing's clocks rest on: `T_ref` (the median
//! four-seam's flight to the contact plane), the swing-start pose's mean and spread (the load
//! pose L), Δ = t_c − T_a, t_s − t_decide (how often and how far the sim's load is cut), `T_a`
//! from a track cut at contact against the full track, and, for saved plays without export E1,
//! t_s and t_c inferred from the encoded bat samples against the swing's own.
//! `cargo run --release -p bb_game --example swing_pose_probe -- [games]`
use bb_math::Vec3;
use bb_observe::PitchType;
use bb_physics::Trajectory;
use bb_physics::contact::BatPose;
use bb_schema::tl;

/// The swing model's contact plane, `SwingTemplate::contact_y0`.
const CONTACT_Y0: f64 = 0.35;

fn row(name: &str, v: &mut Vec<f64>) {
    if v.is_empty() {
        println!("{name:<34} (none)");
        return;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| v[((v.len() - 1) as f64 * p).round() as usize];
    println!("{name:<34} n {:5}  min {:8.4}  p5 {:8.4}  p50 {:8.4}  p95 {:8.4}  max {:8.4}", v.len(), q(0.0), q(0.05), q(0.5), q(0.95), q(1.0));
}

/// Like `row`, in milliseconds.
fn row_ms(name: &str, v: &[f64]) {
    let mut ms: Vec<f64> = v.iter().map(|x| x * 1000.0).collect();
    if ms.is_empty() {
        println!("{name:<34} (none)");
        return;
    }
    ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| ms[((ms.len() - 1) as f64 * p).round() as usize];
    println!("{name:<34} n {:5}  min {:8.3}  p5 {:8.3}  p50 {:8.3}  p95 {:8.3}  max {:8.3}  (ms)", ms.len(), q(0.0), q(0.05), q(0.5), q(0.95), q(1.0));
}

fn mean_sd(v: &[f64]) -> (f64, f64) {
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (m, (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt())
}

/// When the ball's centre reaches y = `CONTACT_Y0`, from the track as known up to `cut` (the
/// whole track if `None`): found on the track if it reaches the plane by then, otherwise
/// extrapolated from the last 30 ms (constant acceleration), as the client will do.
fn t_cross(tr: &Trajectory, cut: Option<f64>) -> Option<f64> {
    let end = cut.map_or(tr.last().t, |c| c.min(tr.last().t));
    let (r_end, v_end) = tr.at(end);
    if r_end.y <= CONTACT_Y0 {
        // On the track: bisect (y falls monotonically through the plane).
        let (mut lo, mut hi) = (tr.first().t, end);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if tr.at(mid).0.y > CONTACT_Y0 { lo = mid } else { hi = mid }
        }
        return Some(0.5 * (lo + hi));
    }
    let (_, v_before) = tr.at(end - 0.03);
    let a = (v_end.y - v_before.y) / 0.03;
    // y(dt) = y + v dt + a dt²/2 = plane.
    let (y, v) = (r_end.y - CONTACT_Y0, v_end.y);
    if a.abs() < 1e-9 {
        return (v < 0.0).then(|| end + y / -v);
    }
    let disc = v * v - 2.0 * a * y;
    if disc < 0.0 {
        return None;
    }
    let s = disc.sqrt();
    [(-v + s) / a, (-v - s) / a].into_iter().filter(|dt| *dt >= 0.0).fold(None, |m: Option<f64>, dt| Some(m.map_or(dt, |m| m.min(dt)))).map(|dt| end + dt)
}

/// The encoded bat samples as (t, knob, axis).
fn bat_samples(bytes: &[u8]) -> Vec<(f64, Vec3, Vec3)> {
    let t = flatbuffers::root::<tl::PlayTimeline>(bytes).unwrap();
    let Some(s) = t.bat().and_then(|b| b.samples()) else { return Vec::new() };
    s.iter()
        .map(|k| {
            let (q, p) = (k.rot(), k.knob());
            let axis = Vec3::new(1.0 - 2.0 * (q.y() * q.y() + q.z() * q.z()) as f64, 2.0 * (q.x() * q.y() + q.w() * q.z()) as f64, 2.0 * (q.x() * q.z() - q.w() * q.y()) as f64);
            (k.t() as f64, Vec3::new(p.x() as f64, p.y() as f64, p.z() as f64), axis)
        })
        .collect()
}

/// Fallback without export E1: the swing's start from the encoded samples, where the load's
/// straight line (one speed, 5 ms apart) gives way to the swing's own curve; `None` when the
/// samples hold no load (the bat goes from the stance to the swing's first pose in one step).
fn infer_t_s(s: &[(f64, Vec3, Vec3)]) -> Option<f64> {
    // The first sample after the stance (index 0 is the stance at t = 0).
    let v = |i: usize| (s[i + 1].1 - s[i].1) / (s[i + 1].0 - s[i].0);
    let mut i = 1;
    // Load samples run until the swing's first sample: find the first slope that departs from
    // the previous slope by more than numerical noise.
    while i + 2 < s.len() {
        if i > 1 && (v(i) - v(i - 1)).norm() > 0.05 {
            return Some(s[i].0);
        }
        i += 1;
    }
    None
}

/// Fallback without export E1: the time of the bat's fastest turning, by a parabola through
/// the three samples round the peak of the turning rate (the sim's bat peaks `bat_peak`, 0.010
/// s at `T_swing` 0.150 and scaled with it, after nominal contact).
fn infer_peak(s: &[(f64, Vec3, Vec3)], t_s: f64) -> Option<f64> {
    let mut best = (0.0, 0usize);
    let rate = |i: usize| (s[i].2.dot(s[i + 1].2).clamp(-1.0, 1.0)).acos() / (s[i + 1].0 - s[i].0);
    for i in 0..s.len() - 1 {
        if s[i].0 >= t_s && rate(i) > best.0 {
            best = (rate(i), i);
        }
    }
    let i = best.1;
    if best.0 == 0.0 || i < 1 || i + 2 >= s.len() {
        return None;
    }
    // Rates at the interval midpoints; vertex of the parabola through three.
    let (r0, r1, r2) = (rate(i - 1), rate(i), rate(i + 1));
    let (t0, t1, t2) = (0.5 * (s[i - 1].0 + s[i].0), 0.5 * (s[i].0 + s[i + 1].0), 0.5 * (s[i + 1].0 + s[i + 2].0));
    let den = (t0 - t1) * (t0 - t2) * (t1 - t2);
    let a = (t2 * (r1 - r0) + t1 * (r0 - r2) + t0 * (r2 - r1)) / den;
    let b = (t2 * t2 * (r0 - r1) + t1 * t1 * (r2 - r0) + t0 * t0 * (r1 - r2)) / den;
    (a < 0.0).then(|| -b / (2.0 * a))
}

fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let names = ["start", "contact-0.05", "contact", "end"];
    let mut knob: Vec<[Vec<f64>; 3]> = (0..4).map(|_| Default::default()).collect();
    let mut axis: Vec<[Vec<f64>; 3]> = (0..4).map(|_| Default::default()).collect();
    let mut far: Vec<Vec<f64>> = vec![Vec::new(); 4];
    let (mut rho, mut cz, mut coff, mut t0s, mut tcs, mut checks, mut n) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), 0, 0);
    // §13 step 1.
    let mut flight_4s = Vec::new(); // four-seam release → contact plane, all pitches
    let mut flight_by_type: Vec<(PitchType, Vec<f64>)> = PitchType::ALL.iter().map(|t| (*t, Vec::new())).collect();
    let (mut delta, mut delta_hit, mut delta_miss, mut gap_load, mut cut_by) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut ta_cut_err, mut ta_cut_err_hit) = (Vec::new(), Vec::new());
    let (mut ts_err, mut tc_err, mut tc_err_checked, mut t_swing) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut starts_rel, mut starts_axis): (Vec<(Vec3, Vec3)>, Vec<Vec3>) = (Vec::new(), Vec::new());
    let (mut load_cut, mut no_load, mut sample_count) = (0u32, 0u32, 0u32);
    let mut peak_after: Vec<f64> = Vec::new();
    let (mut bat_stopped_checked, mut bat_stopped_total) = (0u32, 0u32);
    let mut delta_by_type: Vec<(PitchType, Vec<f64>)> = PitchType::ALL.iter().map(|t| (*t, Vec::new())).collect();
    for seed in 0..games {
        let mut g = bb_game::game::Game::new(seed, true);
        while let Some(step) = g.step_pitch() {
            let rec = &step.record;
            let ta_full = t_cross(&step.pitch_flight, None).filter(|_| rec.plate.is_some() || rec.swing.is_some() || rec.contact_state.is_some());
            if let Some(ta) = ta_full.filter(|_| rec.contact_state.is_none()) {
                // A pitch that reached the plane (the whole track is known): its flight.
                if rec.ty == PitchType::FourSeam {
                    flight_4s.push(ta);
                }
                flight_by_type.iter_mut().find(|(t, _)| *t == rec.ty).unwrap().1.push(ta);
            } else if let Some((tc, _)) = rec.contact_state {
                // A ball hit: the track ends at contact; T_a extrapolated from its last 30 ms.
                if let Some(ta) = t_cross(&step.pitch_flight, Some(tc)) {
                    if rec.ty == PitchType::FourSeam {
                        flight_4s.push(ta);
                    }
                    flight_by_type.iter_mut().find(|(t, _)| *t == rec.ty).unwrap().1.push(ta);
                }
            }
            let (Some(sw), Some((_, at))) = (rec.swing, step.viz.standing.last()) else { continue };
            n += 1;
            checks += sw.t_brake.is_some() as u32;
            // Mirror a left-handed batter (standing at x > 0) into the right-hander's frame.
            let s = if at.x < 0.0 { 1.0 } else { -1.0 };
            let rel = |p: Vec3| Vec3::new(s * (p.x - at.x), p.y - at.y, p.z);
            for (i, t) in [sw.t_start(), sw.t_c - 0.05, sw.t_c, sw.t_c + 0.12].into_iter().enumerate() {
                let k = sw.at(t);
                let (p, a) = (rel(k.knob), Vec3::new(s * k.axis.x, k.axis.y, k.axis.z));
                for (j, c) in [p.x, p.y, p.z].into_iter().enumerate() {
                    knob[i][j].push(c);
                }
                for (j, c) in [a.x, a.y, a.z].into_iter().enumerate() {
                    axis[i][j].push(c);
                }
                far[i].push(p.x.hypot(p.y));
            }
            rho.push(sw.rho);
            cz.push(sw.pivot.z);
            let c = rel(sw.pivot);
            coff.push(c.x.hypot(c.y));
            t0s.push(sw.t_start());
            tcs.push(sw.t_c);
            t_swing.push(sw.t_c - sw.t_start());
            {
                let k = sw.at(sw.t_start());
                starts_rel.push((rel(k.knob), Vec3::new(s * k.axis.x, k.axis.y, k.axis.z)));
                starts_axis.push(Vec3::new(s * k.axis.x, k.axis.y, k.axis.z));
            }
            // The load: how much of the 0.05 s the decision left.
            let gap = sw.t_start() - rec.t_decide;
            gap_load.push(gap);
            if gap < 0.05 {
                load_cut += 1;
                cut_by.push(0.05 - gap);
            }
            no_load += (gap <= 0.0) as u32;
            // Δ = t_c − T_a: T_a from the track as the client sees it (cut at contact on a hit).
            let seen_cut = rec.contact_state.map(|c| c.0);
            if let Some(ta) = t_cross(&step.pitch_flight, seen_cut) {
                delta.push(sw.t_c - ta);
                delta_by_type.iter_mut().find(|(t, _)| *t == rec.ty).unwrap().1.push(sw.t_c - ta);
                if rec.contact_state.is_some() { delta_hit.push(sw.t_c - ta) } else { delta_miss.push(sw.t_c - ta) }
            }
            // T_a from a track cut at nominal contact against the full track, on a swing that
            // misses (the full track exists).
            if rec.contact_state.is_none() && !step.pitch_flight.knots.is_empty() {
                if let (Some(full), Some(cutv)) = (t_cross(&step.pitch_flight, None), t_cross(&step.pitch_flight, Some(sw.t_c))) {
                    ta_cut_err.push(cutv - full);
                }
            } else if let Some((tc, _)) = rec.contact_state {
                // On a hit the track is cut at contact; compare with the same pitch flown on to
                // the plane: contact is within a few cm of the plane, so the extrapolation is
                // short. (The full flight is unknown past contact; use the extrapolation from
                // 30 ms earlier to the plane against the extrapolation from 60 ms earlier.)
                if let (Some(a), Some(b)) = (t_cross(&step.pitch_flight, Some(tc)), t_cross(&step.pitch_flight, Some(tc - 0.03))) {
                    ta_cut_err_hit.push(a - b);
                }
            }
            // Inferred t_s and t_c from the encoded bat samples (what an old saved play has).
            let bytes = bb_game::timeline::encode(&step, &g.st, &g.rosters);
            let samples = bat_samples(&bytes);
            sample_count += samples.len() as u32;
            // The client's fallback: t_s where the load's line ends (known only with a load);
            // t_c from the bat's fastest turning less the template's peak offset (0.010 s at
            // T_swing 0.150, scaled by T_swing, itself t_c - t_s iterated once).
            let t_s_inf = if sw.t_start() - rec.t_decide <= 0.0 {
                None // no load: the bat jumps from the stance to the swing's first pose
            } else {
                infer_t_s(&samples)
            };
            if let Some(ts) = t_s_inf {
                ts_err.push(ts - sw.t_start());
            }
            let ts_used = t_s_inf.unwrap_or_else(|| sw.t_start());
            if let Some(pk) = infer_peak(&samples, ts_used) {
                peak_after.push(pk - sw.t_c);
                // A checked swing's bat has stopped by the end of its track (an unchecked
                // one is still turning at 14% of its contact rate, 0.12 s after): then t_c is
                // t_s + T_SWING_REF (0.150 s), the batter's own T_swing not being known.
                let last = samples.len() - 1;
                let end_rate = (samples[last - 1].2.dot(samples[last].2).clamp(-1.0, 1.0)).acos() / (samples[last].0 - samples[last - 1].0);
                let stopped = end_rate < 1.0;
                let est = if stopped {
                    ts_used + 0.150
                } else {
                    let tsw0 = (pk - 0.010) - ts_used;
                    let tsw = (pk - 0.010 * tsw0 / 0.150) - ts_used;
                    pk - 0.010 * tsw / 0.150
                };
                if sw.t_brake.is_some() { tc_err_checked.push(est - sw.t_c) } else { tc_err.push(est - sw.t_c) }
                bat_stopped_checked += (stopped == sw.t_brake.is_some()) as u32;
                bat_stopped_total += 1;
            }
        }
    }
    println!("{n} swings in {games} games ({checks} checked), {sample_count} bat samples encoded");
    for i in 0..4 {
        for (j, c) in ["x (to the plate)", "y (to the pitcher)", "z"].iter().enumerate() {
            row(&format!("knob {} {c}", names[i]), &mut knob[i][j]);
        }
        row(&format!("knob {} from the spot", names[i]), &mut far[i]);
        for (j, c) in ["x", "y", "z"].iter().enumerate() {
            row(&format!("axis {} {c}", names[i]), &mut axis[i][j]);
        }
    }
    row("hand radius", &mut rho);
    row("arc centre height", &mut cz);
    row("arc centre from the spot", &mut coff);
    row("swing start, s", &mut t0s);
    row("nominal contact, s", &mut tcs);

    println!("\n== §13 step 1 ==");
    println!("T_ref: four-seam, release to the plane y = {CONTACT_Y0} m, all pitches:");
    row("  four-seam flight, s", &mut flight_4s);
    for (t, v) in flight_by_type.iter_mut() {
        row(&format!("  {t:?} flight, s"), v);
    }
    // The load pose L: the mean of the swing-start pose, in the right-hander's frame.
    let col = |f: &dyn Fn(&(Vec3, Vec3)) -> f64| starts_rel.iter().map(f).collect::<Vec<_>>();
    let (kx, ky, kz) = (col(&|p| p.0.x), col(&|p| p.0.y), col(&|p| p.0.z));
    let (ax, ay, az) = (col(&|p| p.1.x), col(&|p| p.1.y), col(&|p| p.1.z));
    println!("\nSwing-start pose (knob from his spot, bat axis), mean ± sd over {} swings:", starts_rel.len());
    for (name, v) in [("knob x (to the plate)", &kx), ("knob y (to the pitcher)", &ky), ("knob z", &kz), ("axis x", &ax), ("axis y", &ay), ("axis z", &az)] {
        let (m, sd) = mean_sd(v);
        println!("  {name:<24} {m:8.4} ± {sd:.4}");
    }
    let l_knob = Vec3::new(mean_sd(&kx).0, mean_sd(&ky).0, mean_sd(&kz).0);
    let l_axis = Vec3::new(mean_sd(&ax).0, mean_sd(&ay).0, mean_sd(&az).0).normalized();
    println!("  L: knob ({:.4}, {:.4}, {:.4}) m   axis ({:.4}, {:.4}, {:.4})", l_knob.x, l_knob.y, l_knob.z, l_axis.x, l_axis.y, l_axis.z);
    let mut d_knob: Vec<f64> = starts_rel.iter().map(|p| (p.0 - l_knob).norm() * 100.0).collect();
    let mut d_axis: Vec<f64> = starts_rel.iter().map(|p| p.1.dot(l_axis).clamp(-1.0, 1.0).acos().to_degrees()).collect();
    row("  distance of a start from L, cm", &mut d_knob);
    row("  angle of its axis from L's, deg", &mut d_axis);

    println!("\nTimes:");
    row("  T_swing = t_c - t_s, s", &mut t_swing);
    row("  t_s - t_decide, s", &mut gap_load);
    println!("  load cut (t_s - t_decide < 0.05 s): {load_cut} of {n} ({:.1}%); no load at all (<= 0): {no_load}", 100.0 * load_cut as f64 / n as f64);
    row("  load cut by (0.05 - gap), s", &mut cut_by);
    row("  delta = t_c - T_a, all, s", &mut delta);
    row("  delta, balls hit, s", &mut delta_hit);
    row("  delta, swings that miss, s", &mut delta_miss);
    for (name, lim) in [("|Δ| ≤ 0.010", 0.010), ("|Δ| ≤ 0.015", 0.015), ("Δ < -0.010", -0.010), ("Δ < -0.025", -0.025), ("Δ > 0.030", 0.030), ("Δ > 0.060", 0.060), ("Δ > 0.075", 0.075)] {
        let c = delta.iter().filter(|d| if name.starts_with('|') { d.abs() <= lim } else if lim < 0.0 { **d < lim } else { **d > lim }).count();
        println!("  {name:<12} {:5} of {} ({:.1}%)", c, delta.len(), 100.0 * c as f64 / delta.len() as f64);
    }
    println!("\nT_a from a track cut at nominal contact, minus T_a from the full track (swings that miss), s:");
    row("  cut - full", &mut ta_cut_err);
    println!("T_a extrapolated from contact on a ball hit, minus the same from 30 ms earlier, s:");
    row("  hit: cut - cut earlier", &mut ta_cut_err_hit);

    println!("\nDelta by pitch type (swings):");
    for (t, v) in delta_by_type.iter_mut() {
        if !v.is_empty() {
            row(&format!("  {t:?}"), v);
        }
    }
    println!("\nInferred times without E1 (inferred - the swing's own):");
    row_ms("  t_s (end of the load's line)", &ts_err);
    row_ms("  t_c, swings not checked", &tc_err);
    row_ms("  t_c, checked swings (t_s + 0.150)", &tc_err_checked);
    println!("  the bat stopped at the end of its track agrees with the swing being checked in {bat_stopped_checked} of {bat_stopped_total}");
    row("  fastest turning minus t_c, s", &mut peak_after);
}
