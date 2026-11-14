//! Comprehensive run-environment diagnosis over 100 games:
//! Q1: K/BB/Foul/Plate discipline by count, pitch type, handedness.
//! Q2: HR / EV / LA / Statcast Barrel% / park & drag counterfactuals.
//! Q3: Ground-ball hit decomposition (H-to-1B, gap geometry, fielder timelines, dives/stretches/throws).
//!
//! Usage: `cargo run --release -p bb_game --example run_env_diag -- [games] [seed0]`

use bb_agents::batter::count_group;
use bb_game::game::{Game, classify_bip};
use bb_game::pa::PitchRecord;
use bb_game::play::{home_positions, run_time};
use bb_math::Vec3;
use bb_math::units::{FT, MPH, radps_to_rpm, rpm_to_radps};
use bb_observe::{Bats, Hand, PitchType};
use bb_physics::bat::{BatProfile, sweet_spot};
use bb_physics::contact::BatPose;
use bb_physics::flight::FlightEnv;
use bb_physics::ground::{Field, Segment, SegmentEnd, batted_path};
use bb_physics::zone::{HALF_WIDTH, Zone, ZoneRule};
use bb_physics::BallState;
use bb_rules::{Event, Input, PaResult};
use bb_stats::scorer::Outcome;

fn pitch_idx(ty: PitchType) -> usize {
    match ty {
        PitchType::FourSeam => 0,
        PitchType::Slider => 1,
        PitchType::Changeup => 2,
        PitchType::Curveball => 3,
        _ => 0,
    }
}

fn hand_pair_idx(throws: Hand, bats: Bats) -> usize {
    let bh = match bats {
        Bats::Right => Hand::Right,
        Bats::Left => Hand::Left,
        Bats::Switch => match throws {
            Hand::Right => Hand::Left,
            Hand::Left => Hand::Right,
        },
    };
    match (throws, bh) {
        (Hand::Right, Hand::Right) => 0,
        (Hand::Left, Hand::Left) => 1,
        (Hand::Right, Hand::Left) => 2,
        (Hand::Left, Hand::Right) => 3,
    }
}

/// Official Statcast Barrel definition (EV >= 98 mph with expanding LA window).
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
        // Upward bound expands ~2 deg per mph up to 50 deg at 115-116 mph.
        let hi_tbl = [
            30.0, 31.0, 33.0, 34.0, 35.0, 37.0, 38.0, 39.0, 40.0, 42.0, 43.0, 44.0, 45.0, 46.0,
            47.0, 48.0, 49.0, 50.0, 50.0,
        ];
        (lo, hi_tbl[(d as usize).min(18)])
    };
    la_deg >= lo && la_deg <= hi
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = q * (sorted.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = (lo + 1).min(sorted.len() - 1);
    let f = pos - lo as f64;
    sorted[lo] * (1.0 - f) + sorted[hi] * f
}

fn mean_sd(xs: &[f64]) -> (f64, f64) {
    if xs.is_empty() {
        return (0.0, 0.0);
    }
    let n = xs.len() as f64;
    let mu = xs.iter().sum::<f64>() / n;
    let var = xs.iter().map(|x| (x - mu) * (x - mu)).sum::<f64>() / n;
    (mu, var.sqrt())
}

#[derive(Default, Clone)]
struct PitchAgg {
    pa: u32,
    k: u32,
    called_k: u32,
    bb: u32,
    hbp: u32,
    hr: u32,
    bip: u32,
    pitches: u32,
    in_zone: u32,
    swings: u32,
    z_swings: u32,
    o_swings: u32,
    contact: u32,
    z_contact: u32,
    o_contact: u32,
    whiff: u32,
    foul: u32,
    cstr: u32,
    first_pitch: u32,
    first_strike: u32,
    bounced: u32,
    check_trig: u32,
    check_called_ball: u32,
    check_called_str: u32,
    check_swing_str: u32,
    check_foul_or_bip: u32,
    // location bands for OOZ
    shadow_out_n: u32,
    shadow_out_sw: u32,
    chase_n: u32,
    chase_sw: u32,
    waste_n: u32,
    waste_sw: u32,
    evs: Vec<f64>,
    las: Vec<f64>,
    barrels: u32,
}

impl PitchAgg {
    fn add_pitch(&mut self, r: &PitchRecord, zone: &Zone, ball_r: f64) {
        self.pitches += 1;
        if r.in_zone {
            self.in_zone += 1;
        }
        if r.bounced {
            self.bounced += 1;
        }
        if r.count.balls == 0 && r.count.strikes == 0 {
            self.first_pitch += 1;
            if !matches!(r.call, bb_game::pa::Call::Ball | bb_game::pa::Call::HitByPitch) {
                self.first_strike += 1;
            }
        }
        if !r.in_zone && !r.bounced {
            if let Some(p) = r.plate {
                let dx = (p.x.abs() - HALF_WIDTH - ball_r).max(0.0);
                let dz = (p.z - zone.top - ball_r).max(zone.bottom - ball_r - p.z).max(0.0);
                let d_cm = dx.hypot(dz) * 100.0;
                if d_cm <= 8.5 {
                    self.shadow_out_n += 1;
                    if r.swung {
                        self.shadow_out_sw += 1;
                    }
                } else if d_cm <= 25.5 {
                    self.chase_n += 1;
                    if r.swung {
                        self.chase_sw += 1;
                    }
                } else {
                    self.waste_n += 1;
                    if r.swung {
                        self.waste_sw += 1;
                    }
                }
            }
        }
        if r.swung {
            self.swings += 1;
            if r.in_zone {
                self.z_swings += 1;
            } else {
                self.o_swings += 1;
            }
            if r.contact {
                self.contact += 1;
                if r.in_zone {
                    self.z_contact += 1;
                } else {
                    self.o_contact += 1;
                }
                if matches!(r.call, bb_game::pa::Call::Foul | bb_game::pa::Call::FoulTip) {
                    self.foul += 1;
                }
            } else {
                self.whiff += 1;
            }
        } else if matches!(r.call, bb_game::pa::Call::CalledStrike) {
            self.cstr += 1;
        }
        if r.check_swing {
            self.check_trig += 1;
            match r.call {
                bb_game::pa::Call::Ball => self.check_called_ball += 1,
                bb_game::pa::Call::CalledStrike => self.check_called_str += 1,
                bb_game::pa::Call::SwingingStrike => self.check_swing_str += 1,
                bb_game::pa::Call::Foul | bb_game::pa::Call::InPlay | bb_game::pa::Call::FoulTip => {
                    self.check_foul_or_bip += 1
                }
                _ => {}
            }
        }
    }

    fn print_row(&self, label: &str) {
        let p = self.pitches.max(1) as f64;
        let pa = self.pa.max(1) as f64;
        let ooz = (self.pitches - self.in_zone).max(1) as f64;
        let iz = self.in_zone.max(1) as f64;
        let sw = self.swings.max(1) as f64;
        let zsw = self.z_swings.max(1) as f64;
        let osw = self.o_swings.max(1) as f64;
        let ct = self.contact.max(1) as f64;
        let (ev_m, _) = mean_sd(&self.evs);
        let (la_m, la_sd) = mean_sd(&self.las);
        let hh = 100.0 * self.evs.iter().filter(|&&e| e >= 95.0).count() as f64 / self.evs.len().max(1) as f64;
        let brl = 100.0 * self.barrels as f64 / self.evs.len().max(1) as f64;
        println!(
            "{label:12} PA {:5} P {:6} P/PA {:.2} | K% {:4.1} BB% {:4.1} HR% {:4.2} BIP% {:4.1} | Z% {:4.1} Sw% {:4.1} ZSw% {:4.1} OSw% {:4.1} | Con% {:4.1} ZCon% {:4.1} OCon% {:4.1} SwStr% {:4.1} CStr% {:4.1} Foul/C% {:4.1} | EV {:4.1} HH% {:4.1} Brl% {:4.1} LA {:4.1}±{:4.1}",
            self.pa,
            self.pitches,
            self.pitches as f64 / pa,
            100.0 * self.k as f64 / pa,
            100.0 * self.bb as f64 / pa,
            100.0 * self.hr as f64 / pa,
            100.0 * self.bip as f64 / pa,
            100.0 * self.in_zone as f64 / p,
            100.0 * self.swings as f64 / p,
            100.0 * self.z_swings as f64 / iz,
            100.0 * self.o_swings as f64 / ooz,
            100.0 * self.contact as f64 / sw,
            100.0 * self.z_contact as f64 / zsw,
            100.0 * self.o_contact as f64 / osw,
            100.0 * self.whiff as f64 / p,
            100.0 * self.cstr as f64 / p,
            100.0 * self.foul as f64 / ct,
            ev_m,
            hh,
            brl,
            la_m,
            la_sd
        );
    }
}

struct BipSample {
    _pair: usize,
    _pitch: usize,
    ev_mph: f64,
    la_deg: f64,
    _spray_deg: f64,
    rpm: f64,
    bat_mph: f64,
    outside_pitch: bool,
    is_hr: bool,
    over_fence: bool,
    clear_ft: f64,
    fence_ft: f64,
    state0: BallState,
    t0: f64,
}

fn fly_carry(env: &FlightEnv, field: &Field, s: BallState, t0: f64) -> (bool, f64, f64) {
    let p = batted_path(env, field, s, t0, |_, _| (0.0, 0.0));
    let mut of = false;
    let mut wall_hit = false;
    let mut max_air_dist_ft = 0.0f64;
    for seg in &p.segments {
        if let Segment::Flight { tr, end } = seg {
            let last = tr.last().s.r;
            let d_ft = last.x.hypot(last.y) / FT;
            max_air_dist_ft = max_air_dist_ft.max(d_ft);
            match end {
                SegmentEnd::OverFence { fair: true } => of = true,
                SegmentEnd::Wall => wall_hit = true,
                _ => {}
            }
            if !matches!(end, SegmentEnd::OverFence { .. }) {
                break; // first flight only for carry / wall check
            }
        }
    }
    let _ = wall_hit;
    (of, max_air_dist_ft, if of { 1.0 } else { 0.0 })
}

fn main() {
    let games: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(100);
    let seed0: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    let env = FlightEnv::default_env();
    let field = Field::default();
    let prof = BatProfile::default_34in();
    let mp = prof.mass_props();
    let uss = sweet_spot(prof.length());
    let zone = Zone::for_batter(1.83, &ZoneRule::default());
    let home = home_positions();

    let mut total_runs: u32 = 0;
    let mut all = PitchAgg::default();
    let mut by_pair: [PitchAgg; 4] = Default::default();
    let mut by_strikes: [PitchAgg; 3] = Default::default();
    let mut by_cg: [PitchAgg; 4] = Default::default();
    let mut by_pitch: [PitchAgg; 4] = Default::default();

    let mut bip_samples: Vec<BipSample> = Vec::new();
    let mut itp_hr_n = 0u32;
    let mut wall_hit_outs = 0u32;
    let mut wall_hit_nonhr = 0u32;
    let mut track_outs_10ft = 0u32;
    let mut fly_outs_n = 0u32;

    // Q3 ground ball trackers
    let mut h1b_all: Vec<f64> = Vec::new();
    let mut h1b_rhb: Vec<f64> = Vec::new();
    let mut h1b_lhb: Vec<f64> = Vec::new();
    let mut gb_n = [0u32; 3]; // 0=through, 1=ifh, 2=out
    let mut gb_outs_dp = 0u32;
    let mut gb_fc_only = 0u32;
    let mut gb_hit_clas = 0u32;
    let mut ifh_dive_tsec: Vec<f64> = Vec::new();
    let mut ifh_nodive_tsec: Vec<f64> = Vec::new();
    let mut ifh_dive_tg2r: Vec<f64> = Vec::new();
    let mut ifh_nodive_tg2r: Vec<f64> = Vec::new();
    let mut ifh_dive_nothrow = 0u32;
    let mut ifh_nodive_nothrow = 0u32;
    let mut ifh_rel_after_1b = 0u32;
    let mut ifh_rel_before_1b_n = 0u32;
    let mut ifh_1b_sec_before_touch = 0u32;
    let mut ifh_1b_touch_late = 0u32;
    let mut ifh_1b_no_sec = 0u32;
    let mut ifh_1b_sec_after_1b = 0u32;
    let mut ifh_throw_other_base = 0u32;
    let mut ifh_throw_bounced = 0u32;
    let mut thru_tot = 0u32;
    let mut thru_reach_def = 0u32;
    let mut thru_reach_bal = 0u32;
    let mut thru_gap_2b1b = 0u32;
    let mut thru_gap_3bss = 0u32;
    let mut routine_fail_thru = 0u32;
    let mut routine_fail_ifh = 0u32;
    let mut routine_fail_dive_miss = 0u32;
    let mut gb_trace_ifh = 0u32;
    let mut gb_trace_rf = 0u32;

    // By LA sub-bins inside <10 deg: <-15, -15..-5, -5..0, 0..5, 5..10
    let mut gb_la_bins = [[0u32; 3]; 5];
    let mut gb_la_ev_sum = [0.0f64; 5];
    let mut gb_la_bounces = [0u32; 5];
    let mut gb_la_n = [0u32; 5];

    // Spray 5-deg bins (-45..+45 => 18 bins)
    let mut gb_spray_all = [[0u32; 3]; 18];
    let mut gb_spray_rhb = [[0u32; 3]; 18];
    let mut gb_spray_lhb = [[0u32; 3]; 18];

    // Official-scorer BIP outcome counts (1B/2B/3B/HR/FC/Out) and line doubles (<= -35/-30, >= +35/+30)
    #[derive(Default, Clone, Copy)]
    struct LineDoubleStats {
        bip: u32,
        hits: u32,
        singles: u32,
        doubles: u32,
        triples: u32,
        hrs: u32,
        fc: u32,
        outs: u32,
        doubles_3b_35: u32,
        doubles_3b_30: u32,
        doubles_1b_35: u32,
        doubles_1b_30: u32,
        doubles_gb_3b_35: u32,
        doubles_gb_3b_30: u32,
        doubles_gb_1b_35: u32,
        doubles_gb_1b_30: u32,
    }
    let mut ld_all = LineDoubleStats::default();
    let mut ld_rhb = LineDoubleStats::default();
    let mut ld_lhb = LineDoubleStats::default();

    // Detailed GB fielding timeline metrics per class (0=through, 1=ifh, 2=out) & routine
    #[derive(Default, Clone)]
    struct GbTimeStats {
        n: u32,
        secured: u32,
        released: u32,
        dives_att: u32,
        dives_caught: u32,
        dives_missed: u32,
        stretch_delay: u32,
        rim_deflect: u32,
        no_throw: u32,
        threw_late: u32,
        hurried: u32,
        off_bal: u32,
        jump_moved: u32,
        jump_wrong: u32,
        t_sec: Vec<f64>,
        t_g2r: Vec<f64>, // release - secure
        t_throw_fly: Vec<f64>, // 1B touch/secure - release
        bounces: Vec<f64>,
        by_pos: [u32; 6],
    }
    let mut gb_ts: [GbTimeStats; 3] = Default::default();
    let mut routine_ts = GbTimeStats::default();
    let mut routine_outs = 0u32;

    for seed in seed0..seed0 + games {
        let mut g = Game::new(seed, true);
        while let Some(st) = g.step_pitch() {
            let batting_side = (st.batter >> 8) as usize;
            let field_side = 1 - batting_side;
            let throws = g.rosters.pitcher_throws(field_side);
            let bats = g.rosters.player(st.batter).map(|p| p.bats).unwrap_or(Bats::Right);
            let pair = hand_pair_idx(throws, bats);
            let bh_is_left = matches!(pair, 1 | 2);
            let pi = pitch_idx(st.record.ty);
            let stk = st.record.count.strikes.min(2) as usize;
            let cg = count_group(st.record.count);

            all.add_pitch(&st.record, &zone, env.ball.radius);
            by_pair[pair].add_pitch(&st.record, &zone, env.ball.radius);
            by_strikes[stk].add_pitch(&st.record, &zone, env.ball.radius);
            by_cg[cg].add_pitch(&st.record, &zone, env.ball.radius);
            by_pitch[pi].add_pitch(&st.record, &zone, env.ball.radius);

            // Track PA ends
            let pa_end = st.rules_events.iter().find_map(|e| {
                if let Event::PlateAppearanceEnded { result, .. } = e {
                    Some(*result)
                } else {
                    None
                }
            });
            if let Some(res_pa) = pa_end {
                let add_pa_res = |agg: &mut PitchAgg| {
                    agg.pa += 1;
                    match res_pa {
                        PaResult::Strikeout => {
                            agg.k += 1;
                            if matches!(st.record.call, bb_game::pa::Call::CalledStrike) {
                                agg.called_k += 1;
                            }
                        }
                        PaResult::Walk => agg.bb += 1,
                        PaResult::HitByPitch => agg.hbp += 1,
                        PaResult::HomeRun => {
                            agg.hr += 1;
                            agg.bip += 1;
                        }
                        PaResult::BattedBall => agg.bip += 1,
                    }
                };
                add_pa_res(&mut all);
                add_pa_res(&mut by_pair[pair]);
                add_pa_res(&mut by_strikes[stk]);
                add_pa_res(&mut by_cg[cg]);
                add_pa_res(&mut by_pitch[pi]);

                if matches!(res_pa, PaResult::HomeRun | PaResult::BattedBall) {
                    if let (Some((ev, la, sp)), Some((t_c, s0)), Some(path)) =
                        (st.record.batted, st.record.contact_state, &st.batted)
                    {
                        let ev_mph = ev / MPH;
                        let la_deg = la.to_degrees();
                        let spray_deg = sp.to_degrees();
                        let brl = is_barrel(ev_mph, la_deg);

                        for agg in [&mut all, &mut by_pair[pair], &mut by_pitch[pi]] {
                            agg.evs.push(ev_mph);
                            agg.las.push(la_deg);
                            if brl {
                                agg.barrels += 1;
                            }
                        }

                        // Wall / OverFence details
                        let mut of = false;
                        let mut wall_hit = false;
                        let mut clear_ft = f64::NAN;
                        let mut fence_ft = f64::NAN;
                        let mut first_land_ft = 0.0f64;
                        for seg in &path.segments {
                            if let Segment::Flight { tr, end } = seg {
                                let last = tr.last().s.r;
                                let d_ft = last.x.hypot(last.y) / FT;
                                match end {
                                    SegmentEnd::OverFence { fair: true } => {
                                        of = true;
                                        let th = last.x.atan2(last.y);
                                        fence_ft = field.fence_at(th) / FT;
                                        clear_ft = (last.z - field.wall_height) / FT;
                                    }
                                    SegmentEnd::Wall => {
                                        wall_hit = true;
                                        first_land_ft = first_land_ft.max(d_ft);
                                    }
                                    SegmentEnd::Ground => {
                                        if first_land_ft == 0.0 {
                                            first_land_ft = d_ft;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        if res_pa == PaResult::HomeRun && !of {
                            itp_hr_n += 1;
                        }
                        let outs_here = st
                            .rules_events
                            .iter()
                            .filter(|e| matches!(e, Event::RunnerOut { .. }))
                            .count();
                        if wall_hit && res_pa != PaResult::HomeRun {
                            wall_hit_nonhr += 1;
                            if outs_here > 0 {
                                wall_hit_outs += 1;
                            }
                        }
                        let batter_fly_out = st.rules_events.iter().any(|e| {
                            matches!(e, Event::RunnerOut { runner, kind: bb_rules::OutKind::FlyOut, .. } if *runner == st.batter)
                        });
                        if batter_fly_out {
                            fly_outs_n += 1;
                            let th = sp;
                            let f_ft = field.fence_at(th) / FT;
                            if first_land_ft >= f_ft - 10.0 {
                                track_outs_10ft += 1;
                            }
                        }

                        // Bat speed & outside pitch check
                        let bat_mph = st
                            .record
                            .swing
                            .map(|sw| {
                                let k = BatPose::at(&sw, t_c);
                                k.point_velocity(k.point(uss), mp.u_cm).norm() / MPH
                            })
                            .unwrap_or(0.0);
                        let outside_pitch = st
                            .record
                            .plate
                            .map(|p| {
                                let side_sign = if bh_is_left { -1.0 } else { 1.0 };
                                p.x * side_sign > 0.05
                            })
                            .unwrap_or(false);

                        bip_samples.push(BipSample {
                            _pair: pair,
                            _pitch: pi,
                            ev_mph,
                            la_deg,
                            _spray_deg: sp.to_degrees(),
                            rpm: radps_to_rpm(s0.w.norm()),
                            bat_mph,
                            outside_pitch,
                            is_hr: res_pa == PaResult::HomeRun,
                            over_fence: of,
                            clear_ft,
                            fence_ft,
                            state0: s0,
                            t0: t_c,
                        });

                        // Official scorer outcome & line-double breakdown
                        if let Some(ruling) = st.scorer_ruling {
                            let is_gb = res_pa == PaResult::BattedBall && la_deg < 10.0;
                            let rec_ld = |s: &mut LineDoubleStats| {
                                s.bip += 1;
                                match ruling.outcome {
                                    Outcome::Hit(1) => { s.hits += 1; s.singles += 1; }
                                    Outcome::Hit(2) => {
                                        s.hits += 1;
                                        s.doubles += 1;
                                        if spray_deg <= -35.0 { s.doubles_3b_35 += 1; if is_gb { s.doubles_gb_3b_35 += 1; } }
                                        if spray_deg <= -30.0 { s.doubles_3b_30 += 1; if is_gb { s.doubles_gb_3b_30 += 1; } }
                                        if spray_deg >= 35.0 { s.doubles_1b_35 += 1; if is_gb { s.doubles_gb_1b_35 += 1; } }
                                        if spray_deg >= 30.0 { s.doubles_1b_30 += 1; if is_gb { s.doubles_gb_1b_30 += 1; } }
                                    }
                                    Outcome::Hit(3) => { s.hits += 1; s.triples += 1; }
                                    Outcome::Hit(4) => { s.hits += 1; s.hrs += 1; }
                                    Outcome::FieldersChoice | Outcome::ReachedOnThrowAward => { s.fc += 1; }
                                    Outcome::Out => { s.outs += 1; }
                                    _ => {}
                                }
                            };
                            rec_ld(&mut ld_all);
                            if bh_is_left { rec_ld(&mut ld_lhb); } else { rec_ld(&mut ld_rhb); }
                        }

                        // H-to-1B check
                        let batter_touched_1b = st.facts.iter().find_map(|f| {
                            if let Input::BaseTouched { t, agent, base: 1, holding_ball: false } = f {
                                (*agent == st.batter).then_some(*t)
                            } else {
                                None
                            }
                        });
                        let batter_forced_or_flyout = st.rules_events.iter().any(|e| {
                            matches!(e, Event::RunnerOut { runner, kind: bb_rules::OutKind::Force | bb_rules::OutKind::FlyOut, .. } if *runner == st.batter)
                        });
                        if let Some(t1b) = batter_touched_1b {
                            if !batter_forced_or_flyout || t1b > t_c {
                                let dt = t1b - t_c;
                                if dt > 2.5 && dt < 6.5 {
                                    h1b_all.push(dt);
                                    if bh_is_left {
                                        h1b_lhb.push(dt);
                                    } else {
                                        h1b_rhb.push(dt);
                                    }
                                }
                            }
                        }

                        // Q3: Ground ball diagnostics (LA < 10 deg, PaResult::BattedBall)
                        if res_pa == PaResult::BattedBall && la_deg < 10.0 {
                            let outs = outs_here;
                            let first_sec = st.facts.iter().find_map(|f| {
                                if let Input::GloveSecured { t, fielder, x, y } = f {
                                    Some((*t, *fielder, *x, *y))
                                } else {
                                    None
                                }
                            });
                            let first_pos = first_sec.map(|(_, f, _, _)| (f & 0x7F) as usize);
                            let class = if outs > 0 {
                                2
                            } else if first_pos.map(|p| p >= 7).unwrap_or(true) {
                                0
                            } else {
                                1
                            };
                            gb_n[class] += 1;
                            if outs >= 2 {
                                gb_outs_dp += 1;
                            }
                            let (rf, hit) = classify_bip(st.batter, &st.rules_events, &st.facts);
                            if hit {
                                gb_hit_clas += 1;
                            }
                            if rf && !hit {
                                gb_fc_only += 1;
                            }

                            // LA sub-bin
                            let lbin = if la_deg < -15.0 {
                                0
                            } else if la_deg < -5.0 {
                                1
                            } else if la_deg < 0.0 {
                                2
                            } else if la_deg < 5.0 {
                                3
                            } else {
                                4
                            };
                            gb_la_n[lbin] += 1;
                            gb_la_bins[lbin][class] += 1;
                            gb_la_ev_sum[lbin] += ev_mph;
                            let t_cut = first_sec.map(|x| x.0).unwrap_or(f64::INFINITY);
                            let b_cnt = path.bounces.iter().filter(|b| b.0 <= t_cut).count() as u32;
                            gb_la_bounces[lbin] += b_cnt;

                            // Spray bin (-45..+45 in 5-deg bins)
                            let sbin = (((spray_deg + 45.0) / 5.0).floor() as isize).clamp(0, 17) as usize;
                            gb_spray_all[sbin][class] += 1;
                            if bh_is_left {
                                gb_spray_lhb[sbin][class] += 1;
                            } else {
                                gb_spray_rhb[sbin][class] += 1;
                            }

                            let mut play_home = home;
                            for &(id, p) in &st.viz.standing {
                                if (id & 0x80) != 0 {
                                    let idx = (id & 0x7F) as usize;
                                    if (1..=9).contains(&idx) {
                                        play_home[idx - 1] = p;
                                    }
                                }
                            }

                            // Routine check (<95 mph & <= 3 m of home pos P,1B,2B,3B,SS)
                            let routine = ev_mph < 95.0
                                && (0..6).filter(|&i| i != 1).any(|i| {
                                    let h = play_home[i];
                                    let mut tt = path.t_start();
                                    let mut near = false;
                                    while tt < path.t_end() && !near {
                                        let p = path.at(tt).0;
                                        near = p.z <= 2.4 && p.x.hypot(p.y) <= 45.0 && (p.x - h.x).hypot(p.y - h.y) <= 3.0;
                                        tt += 0.01;
                                    }
                                    near
                                });
                            if routine && outs > 0 {
                                routine_outs += 1;
                            }

                            let update_ts = |ts: &mut GbTimeStats| {
                                ts.n += 1;
                                ts.bounces.push(b_cnt as f64);
                                if let Some((ts_sec, fld, _, _)) = first_sec {
                                    let pos = (fld & 0x7F) as usize;
                                    if pos <= 6 {
                                        ts.secured += 1;
                                        if pos >= 1 {
                                            ts.by_pos[pos - 1] += 1;
                                        }
                                        let rel_idx = st.viz.released.iter().position(|(_, id)| *id == fld);
                                        if let Some(ri) = rel_idx {
                                            ts.released += 1;
                                            let tr = st.viz.released[ri].0;
                                            ts.t_g2r.push(tr - ts_sec);
                                            if let Some(&(hur, offb)) = st.viz.hurry.get(ri) {
                                                if hur > 0.0 {
                                                    ts.hurried += 1;
                                                }
                                                if offb {
                                                    ts.off_bal += 1;
                                                }
                                            }
                                            // Look for 1B secure or BaseTouched(1, holding_ball=true) shortly after release
                                            let t_at_1b = st.facts.iter().filter_map(|f| match f {
                                                Input::GloveSecured { t, fielder, .. } if *t > tr && *t < tr + 2.5 && (*fielder & 0x7F) == 3 => Some(*t),
                                                Input::BaseTouched { t, base: 1, holding_ball: true, .. } if *t >= tr && *t < tr + 2.5 => Some(*t),
                                                _ => None,
                                            }).fold(f64::INFINITY, f64::min);
                                            if t_at_1b.is_finite() {
                                                ts.t_throw_fly.push(t_at_1b - tr);
                                            }
                                        } else {
                                            ts.no_throw += 1;
                                        }
                                        ts.t_sec.push(ts_sec - t_c);
                                        // Stretch / dive detection
                                        let dove_sec = st.viz.dives.iter().any(|(td, id, _, _)| *id == fld && *td <= ts_sec + 1e-6);
                                        if dove_sec {
                                            ts.dives_caught += 1;
                                        } else if let Some(ri) = rel_idx {
                                            // Infield transfer is 0.65s; if g2r > 0.90 without dive, check stretch (~0.3s extra)
                                            if st.viz.released[ri].0 - ts_sec > 0.85 {
                                                ts.stretch_delay += 1;
                                            }
                                        }
                                    }
                                    if let (Some(ri), Some(t1b)) = (rel_idx_check(&st.viz.released, fld), batter_touched_1b) {
                                        if t1b < t_c + 10.0 {
                                            let tr = st.viz.released[ri].0;
                                            let t_at_1b = st.facts.iter().filter_map(|f| match f {
                                                Input::BaseTouched { t, base: 1, holding_ball: true, .. } if *t >= tr && *t < tr + 2.5 => Some(*t),
                                                _ => None,
                                            }).fold(f64::INFINITY, f64::min);
                                            if t_at_1b.is_finite() && t1b < t_at_1b {
                                                ts.threw_late += 1;
                                            }
                                        }
                                    }
                                }
                                let if_dives = st.viz.dives.iter().filter(|(_, id, _, _)| (*id & 0x7F) <= 6).count() as u32;
                                ts.dives_att += if_dives;
                                if first_sec.is_none() || first_pos.map(|p| p >= 7).unwrap_or(false) {
                                    ts.dives_missed += if_dives;
                                }
                                // Rim deflection: check if any non-ground BallTouched or GloveSecured? Actually live.rs doesn't emit BallTouched on rim; omit proxy
                                if first_sec.is_none() && if_dives > 0 {
                                    ts.rim_deflect += 1;
                                }
                                // Jump check: for nearest infielder at start to ball's y=play_home[i].y path
                                let (b_v, _) = (s0.v, s0.r);
                                if let Some((ni, _, _)) = (2..6).filter_map(|i| {
                                    let hp = play_home[i];
                                    let d = hp.x.hypot(hp.y);
                                    // approximate lateral sign of ball relative to fielder at fielder dist
                                    let b_dir = Vec3::new(b_v.x, b_v.y, 0.0).normalized();
                                    let cross = hp.x * b_dir.y - hp.y * b_dir.x; // lateral offset
                                    Some((i, cross.abs(), (d, cross)))
                                }).min_by(|a, b| a.1.total_cmp(&b.1)) {
                                    let id_pos = ni as u16 + 1;
                                    if let Some((_, samples)) = st.viz.actors.iter().find(|(id, _)| (*id & 0x80 != 0) && (*id & 0x7F) == id_pos) {
                                        if let Some(p_25) = samples.iter().min_by(|a, b| (a.0 - (t_c + 0.25)).abs().total_cmp(&(b.0 - (t_c + 0.25)).abs())) {
                                            let dp = p_25.1 - play_home[ni];
                                            if dp.x.hypot(dp.y) > 0.05 {
                                                ts.jump_moved += 1;
                                                // dot with vector from fielder to ball's direction ray closest point
                                                let b2 = Vec3::new(b_v.x, b_v.y, 0.0).normalized();
                                                let proj = b2 * (play_home[ni].x * b2.x + play_home[ni].y * b2.y);
                                                let want = proj - play_home[ni];
                                                if dp.x * want.x + dp.y * want.y < 0.0 {
                                                    ts.jump_wrong += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            };
                            update_ts(&mut gb_ts[class]);
                            if class == 0 {
                                thru_tot += 1;
                                if spray_deg >= 13.0 && spray_deg <= 38.0 {
                                    thru_gap_2b1b += 1;
                                } else if spray_deg >= -36.0 && spray_deg <= -13.0 {
                                    thru_gap_3bss += 1;
                                }
                                let p_pol = |d_ft: f64, deg: f64| {
                                    let d = d_ft * FT;
                                    let a = deg.to_radians();
                                    Vec3::new(d * a.sin(), d * a.cos(), 0.0)
                                };
                                let bal_pos = [
                                    p_pol(110.0, 32.0),  // 1B shifted slightly off line
                                    p_pol(150.0, 14.5),  // 2B slightly wider
                                    p_pol(115.0, -32.0), // 3B off line
                                    p_pol(150.0, -14.5), // SS slightly wider
                                ];
                                let def_pos = [play_home[2], play_home[3], play_home[4], play_home[5]];
                                let can_reach = |pos_list: &[Vec3; 4]| {
                                    let mut tt = path.t_start() + 0.25;
                                    while tt < path.t_end() {
                                        let p = path.at(tt).0;
                                        if p.z <= 2.4 && p.x.hypot(p.y) <= 52.0 {
                                            let dt = tt - t_c - 0.25;
                                            for &fp in pos_list {
                                                let d = (p.x - fp.x).hypot(p.y - fp.y);
                                                let needed = (d - 1.0).max(0.0);
                                                if dt >= run_time(needed, 7.6, 0.9) {
                                                    return true;
                                                }
                                            }
                                        }
                                        tt += 0.02;
                                    }
                                    false
                                };
                                if can_reach(&def_pos) { thru_reach_def += 1; }
                                if can_reach(&bal_pos) { thru_reach_bal += 1; }
                            }
                            if class == 1 {
                                if let Some((ts_sec, fld, sx, sy)) = first_sec {
                                    let dt_sec = ts_sec - t_c;
                                    let dove = st.viz.dives.iter().any(|(td, id, _, _)| *id == fld && *td <= ts_sec + 1e-6);
                                    let rel = st.viz.released.iter().find(|(_, id)| *id == fld).map(|x| x.0);
                                    if dove {
                                        ifh_dive_tsec.push(dt_sec);
                                        if let Some(tr) = rel {
                                            ifh_dive_tg2r.push(tr - ts_sec);
                                        } else {
                                            ifh_dive_nothrow += 1;
                                        }
                                    } else {
                                        ifh_nodive_tsec.push(dt_sec);
                                        if let Some(tr) = rel {
                                            ifh_nodive_tg2r.push(tr - ts_sec);
                                        } else {
                                            ifh_nodive_nothrow += 1;
                                        }
                                    }
                                    if let (Some(tr), Some(t1b)) = (rel, batter_touched_1b) {
                                        if tr > t1b {
                                            ifh_rel_after_1b += 1;
                                        } else {
                                            ifh_rel_before_1b_n += 1;
                                            let next_sec = st.facts.iter().find_map(|f| match f {
                                                Input::GloveSecured { t, fielder, .. } if *t > tr => Some((*t, *fielder & 0x7F)),
                                                _ => None,
                                            });
                                            let bounced = next_sec.is_some_and(|(tn, _)| {
                                                st.viz.batted.iter().any(|s| s[0] > tr && s[0] < tn && s[3] < 0.1)
                                            });
                                            if bounced {
                                                ifh_throw_bounced += 1;
                                            }
                                            let t_3_sec = st.facts.iter().find_map(|f| match f {
                                                Input::GloveSecured { t, fielder, .. } if *t > tr && (*fielder & 0x7F) == 3 => Some(*t),
                                                _ => None,
                                            });
                                            let t_1_hold = st.facts.iter().find_map(|f| match f {
                                                Input::BaseTouched { t, base: 1, holding_ball: true, .. } if *t >= tr => Some(*t),
                                                _ => None,
                                            });
                                            if let Some((_, pos_to)) = next_sec {
                                                if pos_to != 3 && t_1_hold.is_none() {
                                                    ifh_throw_other_base += 1;
                                                }
                                            }
                                            match (t_3_sec, t_1_hold) {
                                                (Some(ts3), Some(th1)) if ts3 <= t1b && th1 > t1b => ifh_1b_sec_before_touch += 1,
                                                (Some(ts3), None) if ts3 <= t1b => ifh_1b_sec_before_touch += 1,
                                                (Some(ts3), _) if ts3 > t1b => ifh_1b_sec_after_1b += 1,
                                                (_, Some(th1)) if th1 > t1b => ifh_1b_touch_late += 1,
                                                (None, None) => ifh_1b_no_sec += 1,
                                                _ => {}
                                            }
                                        }
                                    }
                                    if gb_trace_ifh < 8 {
                                        gb_trace_ifh += 1;
                                        let brief_facts: Vec<String> = st.facts.iter().filter_map(|f| match f {
                                            Input::GloveSecured { t, fielder, x, y } => Some(format!("Sec({:.2},pos{},{:.1},{:.1})", t - t_c, fielder & 0x7F, x, y)),
                                            Input::BaseTouched { t, agent, base, holding_ball } => Some(format!("Touch({:.2},b={},hold={},bat={})", t - t_c, base, holding_ball, *agent == st.batter)),
                                            _ => None,
                                        }).collect();
                                        println!("  [IFH sample] s={seed} p={} pos={} EV={:.1} LA={:.1} sp={:.1} at=({:.1},{:.1}) dove={} t_sec={:.2} rel={:?} hurry={:?} t_1b={:?} | facts={:?} | ev={:?}",
                                            st.play, fld & 0x7F, ev_mph, la_deg, spray_deg, sx, sy, dove, dt_sec,
                                            rel.map(|x| ((x - t_c)*100.0).round()/100.0), st.viz.hurry,
                                            batter_touched_1b.map(|x| ((x - t_c)*100.0).round()/100.0), brief_facts, st.rules_events);
                                    }
                                }
                            }
                            if routine {
                                update_ts(&mut routine_ts);
                                if outs == 0 {
                                    if class == 0 {
                                        routine_fail_thru += 1;
                                    } else {
                                        routine_fail_ifh += 1;
                                    }
                                    if st.viz.dives.iter().any(|(_, id, _, _)| (*id & 0x7F) <= 6) {
                                        routine_fail_dive_miss += 1;
                                    }
                                    if gb_trace_rf < 8 {
                                        gb_trace_rf += 1;
                                        let dove_n = st.viz.dives.iter().filter(|(_, id, _, _)| (*id & 0x7F) <= 6).count();
                                        let brief_facts: Vec<String> = st.facts.iter().filter_map(|f| match f {
                                            Input::GloveSecured { t, fielder, x, y } => Some(format!("Sec({:.2},pos{},{:.1},{:.1})", t - t_c, fielder & 0x7F, x, y)),
                                            Input::BaseTouched { t, agent, base, holding_ball } => Some(format!("Touch({:.2},b={},hold={},bat={})", t - t_c, base, holding_ball, *agent == st.batter)),
                                            _ => None,
                                        }).collect();
                                        println!("  [RoutineFail sample] s={seed} p={} class={class} EV={:.1} LA={:.1} sp={:.1} first={:?} dives={dove_n} rel={:?} hurry={:?} | facts={:?}",
                                            st.play, ev_mph, la_deg, spray_deg,
                                            first_sec.map(|(t, f, x, y)| (f & 0x7F, ((t - t_c)*100.0).round()/100.0, (x*10.0).round()/10.0, (y*10.0).round()/10.0)),
                                            st.viz.released.iter().map(|(t, f)| (f & 0x7F, ((t - t_c)*100.0).round()/100.0)).collect::<Vec<_>>(),
                                            st.viz.hurry, brief_facts);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let res = g.finish();
        total_runs += res.runs[0] as u32 + res.runs[1] as u32;
    }

    println!("=== 100-GAME BASELINE SUMMARY ({games} games, {seed0}..{}) ===", seed0 + games);
    println!("Runs/team-game: {:.3}", total_runs as f64 / (2 * games) as f64);
    println!("Called-K share of K: {:.1}% ({}/{})", 100.0 * all.called_k as f64 / all.k.max(1) as f64, all.called_k, all.k);
    println!("Bounced pitches: {} ({:.1}% of pitches, {:.1}% of OOZ)", all.bounced, 100.0 * all.bounced as f64 / all.pitches.max(1) as f64, 100.0 * all.bounced as f64 / (all.pitches - all.in_zone).max(1) as f64);
    println!("F-Strike%: {:.1}% ({}/{})", 100.0 * all.first_strike as f64 / all.first_pitch.max(1) as f64, all.first_strike, all.first_pitch);
    println!("OOZ non-bounced swing% by edge dist: shadow(0-8.5cm) {:.1}% ({}/{}), chase(8.5-25.5cm) {:.1}% ({}/{}), waste(>25.5cm) {:.1}% ({}/{})",
        100.0 * all.shadow_out_sw as f64 / all.shadow_out_n.max(1) as f64, all.shadow_out_sw, all.shadow_out_n,
        100.0 * all.chase_sw as f64 / all.chase_n.max(1) as f64, all.chase_sw, all.chase_n,
        100.0 * all.waste_sw as f64 / all.waste_n.max(1) as f64, all.waste_sw, all.waste_n);
    println!("Check swings triggered: {} -> called ball {}, called strike {}, swinging K {}, foul/BIP {}",
        all.check_trig, all.check_called_ball, all.check_called_str, all.check_swing_str, all.check_foul_or_bip);
    println!();
    all.print_row("ALL");
    println!("--- By Strikes ---");
    for s in 0..3 {
        by_strikes[s].print_row(&format!("{s} strikes"));
    }
    println!("--- By Count Group ---");
    for (g, lbl) in ["CG0 behind", "CG1 even", "CG2 ahead", "CG3 2-str"].iter().enumerate() {
        by_cg[g].print_row(lbl);
    }
    println!("--- By Pitch Type ---");
    for (p, lbl) in ["FourSeam", "Slider", "Changeup", "Curveball"].iter().enumerate() {
        by_pitch[p].print_row(lbl);
    }
    println!("--- By Handedness Pair (Pitcher/Batter) ---");
    for (i, lbl) in ["R/R", "L/L", "R/L", "L/R"].iter().enumerate() {
        let share = 100.0 * by_pair[i].pa as f64 / all.pa.max(1) as f64;
        println!("  {lbl} PA share: {share:.1}% ({})", by_pair[i].pa);
        by_pair[i].print_row(lbl);
    }

    // ---- Q2: EV, LA, Barrels, HR & Trajectory Sweep
    println!("\n=== Q2: BATTED BALL, EV, LA, BARRELS & HOME RUNS ===");
    let n_bip = bip_samples.len();
    let mut evs: Vec<f64> = bip_samples.iter().map(|b| b.ev_mph).collect();
    let mut las: Vec<f64> = bip_samples.iter().map(|b| b.la_deg).collect();
    let mut bats: Vec<f64> = bip_samples.iter().filter(|b| b.bat_mph > 0.0).map(|b| b.bat_mph).collect();
    let mut bats_out: Vec<f64> = bip_samples.iter().filter(|b| b.bat_mph > 0.0 && b.outside_pitch).map(|b| b.bat_mph).collect();
    let mut bats_in: Vec<f64> = bip_samples.iter().filter(|b| b.bat_mph > 0.0 && !b.outside_pitch).map(|b| b.bat_mph).collect();
    let mut hr_bats: Vec<f64> = bip_samples.iter().filter(|b| b.is_hr && b.bat_mph > 0.0).map(|b| b.bat_mph).collect();
    evs.sort_by(|a, b| a.total_cmp(b));
    las.sort_by(|a, b| a.total_cmp(b));
    bats.sort_by(|a, b| a.total_cmp(b));
    bats_out.sort_by(|a, b| a.total_cmp(b));
    bats_in.sort_by(|a, b| a.total_cmp(b));
    hr_bats.sort_by(|a, b| a.total_cmp(b));
    let (ev_m, ev_sd) = mean_sd(&evs);
    let (la_m, la_sd) = mean_sd(&las);
    println!("Fair BIP: {n_bip} | EV mean {ev_m:.2} SD {ev_sd:.2} mph | q10 {:.1} q25 {:.1} q50 {:.1} q75 {:.1} q90 {:.1} q95 {:.1} q99 {:.1} max {:.1}",
        quantile(&evs, 0.10), quantile(&evs, 0.25), quantile(&evs, 0.50), quantile(&evs, 0.75), quantile(&evs, 0.90), quantile(&evs, 0.95), quantile(&evs, 0.99), evs.last().copied().unwrap_or(0.0));
    println!("EV >=95 (HardHit): {:.1}%, >=100: {:.1}%, >=105: {:.1}%, >=110: {:.1}%",
        100.0 * evs.iter().filter(|&&x| x >= 95.0).count() as f64 / n_bip.max(1) as f64,
        100.0 * evs.iter().filter(|&&x| x >= 100.0).count() as f64 / n_bip.max(1) as f64,
        100.0 * evs.iter().filter(|&&x| x >= 105.0).count() as f64 / n_bip.max(1) as f64,
        100.0 * evs.iter().filter(|&&x| x >= 110.0).count() as f64 / n_bip.max(1) as f64);
    println!("LA mean {la_m:.2} SD {la_sd:.2} deg | q10 {:.1} q25 {:.1} q50 {:.1} q75 {:.1} q90 {:.1}",
        quantile(&las, 0.10), quantile(&las, 0.25), quantile(&las, 0.50), quantile(&las, 0.75), quantile(&las, 0.90));
    let swsp = bip_samples.iter().filter(|b| (8.0..=32.0).contains(&b.la_deg)).count();
    let la15_40 = bip_samples.iter().filter(|b| (15.0..=40.0).contains(&b.la_deg)).count();
    let gb_c = bip_samples.iter().filter(|b| b.la_deg < 10.0).count();
    let ld_c = bip_samples.iter().filter(|b| (10.0..25.0).contains(&b.la_deg)).count();
    let fb_c = bip_samples.iter().filter(|b| (25.0..50.0).contains(&b.la_deg)).count();
    let pu_c = bip_samples.iter().filter(|b| b.la_deg >= 50.0).count();
    println!("LA zones: SweetSpot(8-32°): {:.1}% | 15-40°: {:.1}% | GB(<10): {:.1}% | LD(10-25): {:.1}% | FB(25-50): {:.1}% | PU(>=50): {:.1}%",
        100.0 * swsp as f64 / n_bip as f64, 100.0 * la15_40 as f64 / n_bip as f64,
        100.0 * gb_c as f64 / n_bip as f64, 100.0 * ld_c as f64 / n_bip as f64,
        100.0 * fb_c as f64 / n_bip as f64, 100.0 * pu_c as f64 / n_bip as f64);

    let hrs: Vec<&BipSample> = bip_samples.iter().filter(|b| b.is_hr).collect();
    let of_hrs: Vec<&BipSample> = hrs.iter().copied().filter(|b| b.over_fence).collect();
    let brls: Vec<&BipSample> = bip_samples.iter().filter(|b| is_barrel(b.ev_mph, b.la_deg)).collect();
    let brl_hr = brls.iter().filter(|b| b.is_hr).count();
    let non_brl_of_hr = of_hrs.iter().filter(|b| !is_barrel(b.ev_mph, b.la_deg)).count();
    println!("Total HRs: {} ({:.2}% PA, {:.2}% BIP) = {} over-fence + {} inside-the-park",
        hrs.len(), 100.0 * hrs.len() as f64 / all.pa.max(1) as f64, 100.0 * hrs.len() as f64 / n_bip.max(1) as f64, of_hrs.len(), itp_hr_n);
    println!("Barrels: {} ({:.2}% BIP, {:.2}% PA) | HR/Barrel: {:.1}% ({}/{}) | non-Barrel OF HRs: {} | Wall-hit non-HRs: {} (outs {}) | Fly-outs <=10ft of wall: {}/{}",
        brls.len(), 100.0 * brls.len() as f64 / n_bip.max(1) as f64, 100.0 * brls.len() as f64 / all.pa.max(1) as f64,
        100.0 * brl_hr as f64 / brls.len().max(1) as f64, brl_hr, brls.len(), non_brl_of_hr, wall_hit_nonhr, wall_hit_outs, track_outs_10ft, fly_outs_n);
    println!("Bat speed (fair BIP): mean {:.1}, p50 {:.1}, p90 {:.1} | inside p50 {:.1}/p90 {:.1}, outside p50 {:.1}/p90 {:.1} | HR mean {:.1}, p50 {:.1}, p90 {:.1}",
        mean_sd(&bats).0, quantile(&bats, 0.5), quantile(&bats, 0.9),
        quantile(&bats_in, 0.5), quantile(&bats_in, 0.9),
        quantile(&bats_out, 0.5), quantile(&bats_out, 0.9),
        mean_sd(&hr_bats).0, quantile(&hr_bats, 0.5), quantile(&hr_bats, 0.9));

    println!("Over-Fence HRs by LA bin:");
    for (lo, hi) in [(-90.0, 15.0), (15.0, 20.0), (20.0, 25.0), (25.0, 30.0), (30.0, 35.0), (35.0, 40.0), (40.0, 90.0)] {
        let sub: Vec<&&BipSample> = of_hrs.iter().filter(|b| b.la_deg >= lo && b.la_deg < hi).collect();
        let evs_s: Vec<f64> = sub.iter().map(|b| b.ev_mph).collect();
        let rpms_s: Vec<f64> = sub.iter().map(|b| b.rpm).collect();
        let cls_s: Vec<f64> = sub.iter().map(|b| b.clear_ft).collect();
        let all_bin = bip_samples.iter().filter(|b| b.la_deg >= lo && b.la_deg < hi).count();
        println!("  LA {:4.0}..{:4.0}: OF-HR {:3} / {:4} BIP ({:4.1}%) | mean EV {:5.1} min {:5.1} | spin {:4.0} rpm | wall clear {:4.1} ft",
            lo, hi, sub.len(), all_bin, 100.0 * sub.len() as f64 / all_bin.max(1) as f64,
            mean_sd(&evs_s).0, evs_s.iter().copied().fold(f64::INFINITY, f64::min), mean_sd(&rpms_s).0, mean_sd(&cls_s).0);
    }
    println!("Over-Fence HRs by EV bin:");
    for (lo, hi) in [(0.0, 95.0), (95.0, 98.0), (98.0, 100.0), (100.0, 105.0), (105.0, 110.0), (110.0, 200.0)] {
        let sub: Vec<&&BipSample> = of_hrs.iter().filter(|b| b.ev_mph >= lo && b.ev_mph < hi).collect();
        let all_bin = bip_samples.iter().filter(|b| b.ev_mph >= lo && b.ev_mph < hi).count();
        let las_s: Vec<f64> = sub.iter().map(|b| b.la_deg).collect();
        let f_s: Vec<f64> = sub.iter().map(|b| b.fence_ft).collect();
        let c_s: Vec<f64> = sub.iter().map(|b| b.clear_ft).collect();
        println!("  EV {:3.0}..{:3.0}: OF-HR {:3} / {:4} BIP ({:4.1}%) | mean LA {:4.1} | fence {:5.1} ft | clear {:4.1} ft",
            lo, hi, sub.len(), all_bin, 100.0 * sub.len() as f64 / all_bin.max(1) as f64,
            mean_sd(&las_s).0, mean_sd(&f_s).0, mean_sd(&c_s).0);
    }
    let mut cls_sorted: Vec<f64> = of_hrs.iter().map(|b| b.clear_ft).collect();
    cls_sorted.sort_by(|a, b| a.total_cmp(b));
    let mut f_sorted: Vec<f64> = of_hrs.iter().map(|b| b.fence_ft).collect();
    f_sorted.sort_by(|a, b| a.total_cmp(b));
    println!("OF-HR wall clearance (above 8ft): p10 {:.1} ft, p25 {:.1}, p50 {:.1}, p75 {:.1} | fence dist p10 {:.1}, p50 {:.1}, p90 {:.1}",
        quantile(&cls_sorted, 0.1), quantile(&cls_sorted, 0.25), quantile(&cls_sorted, 0.5), quantile(&cls_sorted, 0.75),
        quantile(&f_sorted, 0.1), quantile(&f_sorted, 0.5), quantile(&f_sorted, 0.9));

    // Carry / Aero / Fence Counterfactual on actual air BIP
    let air_cands: Vec<&BipSample> = bip_samples.iter().filter(|b| b.ev_mph >= 88.0 && b.la_deg >= 8.0 && b.la_deg <= 55.0).collect();
    let count_of = |e: &FlightEnv, f: &Field, ev_delta_mph: f64| -> usize {
        air_cands.iter().filter(|b| {
            let mut s = b.state0;
            if ev_delta_mph != 0.0 {
                let n = s.v.norm();
                let target = (n + ev_delta_mph * MPH).max(1.0);
                s.v = s.v * (target / n);
            }
            fly_carry(e, f, s, b.t0).0
        }).count()
    };
    let mut env_isa = env;
    env_isa.rho = 1.225;
    let mut env_hi_drag1 = env;
    env_hi_drag1.aero.cd0 = 0.330;
    let mut env_hi_drag2 = env;
    env_hi_drag2.aero.cd0 = 0.350;
    let env_calc = FlightEnv::calculator();
    let field_deep = Field { fence_lf: 340.0 * FT, fence_cf: 410.0 * FT, fence_rf: 340.0 * FT, ..field };
    let field_tall12 = Field { wall_height: 12.0 * FT, ..field };
    let field_tall16 = Field { wall_height: 16.0 * FT, ..field };
    println!("Trajectory counterfactuals on {} air-ball candidates (baseline OF-HRs = {}):", air_cands.len(), count_of(&env, &field, 0.0));
    println!("  ISA sea-level rho=1.225 (vs 1.1938): {} OF-HRs", count_of(&env_isa, &field, 0.0));
    println!("  Calculator (no spin decay):           {} OF-HRs", count_of(&env_calc, &field, 0.0));
    println!("  cd0=0.330 (+9.7% base drag):          {} OF-HRs", count_of(&env_hi_drag1, &field, 0.0));
    println!("  cd0=0.350 (+16.4% base drag):         {} OF-HRs", count_of(&env_hi_drag2, &field, 0.0));
    println!("  Fence 340/410/340 (+10ft deep):       {} OF-HRs", count_of(&env, &field_deep, 0.0));
    println!("  Wall 12ft (+4ft tall):                {} OF-HRs", count_of(&env, &field_tall12, 0.0));
    println!("  Wall 16ft (+8ft tall):                {} OF-HRs", count_of(&env, &field_tall16, 0.0));
    println!("  Exit velo -2.0 mph:                   {} OF-HRs", count_of(&env, &field, -2.0));
    println!("  Exit velo -4.0 mph:                   {} OF-HRs", count_of(&env, &field, -4.0));

    // Standard 100 mph, 28 deg, 2000 rpm test carry
    let test_s = |sp: f64| {
        let la = 28.0f64.to_radians();
        let v = 100.0 * MPH;
        BallState {
            r: Vec3::new(0.0, 0.5, 0.9),
            v: Vec3::new(0.0, v * la.cos(), v * la.sin()),
            w: Vec3::new(rpm_to_radps(sp), 0.0, 0.0),
        }
    };
    let (_, d_def, _) = fly_carry(&env, &Field { fence_lf: 900.0 * FT, fence_cf: 900.0 * FT, fence_rf: 900.0 * FT, ..field }, test_s(2000.0), 0.0);
    let (_, d_isa, _) = fly_carry(&env_isa, &Field { fence_lf: 900.0 * FT, fence_cf: 900.0 * FT, fence_rf: 900.0 * FT, ..field }, test_s(2000.0), 0.0);
    let (_, d_calc, _) = fly_carry(&env_calc, &Field { fence_lf: 900.0 * FT, fence_cf: 900.0 * FT, fence_rf: 900.0 * FT, ..field }, test_s(2000.0), 0.0);
    println!("Nathan test ball (100 mph, 28°, 2000 rpm carry): default_env {:.1} ft | rho=1.225 {:.1} ft | calculator(no decay) {:.1} ft",
        d_def, d_isa, d_calc);

    // ---- Q3: Ground Ball Diagnosis
    println!("\n=== Q3: GROUND BALLS (LA < 10 deg, PaResult::BattedBall) ===");
    let gb_tot = gb_n.iter().sum::<u32>().max(1);
    println!("Total GB: {gb_tot} -> Through: {} ({:.1}%), IFH: {} ({:.1}%), Outs: {} ({:.1}%, 2+outs {}), Total Hit% (through+IFH): {:.3}, classify_bip hit {:.3}, FC-only {:.3}",
        gb_n[0], 100.0 * gb_n[0] as f64 / gb_tot as f64,
        gb_n[1], 100.0 * gb_n[1] as f64 / gb_tot as f64,
        gb_n[2], 100.0 * gb_n[2] as f64 / gb_tot as f64, gb_outs_dp,
        (gb_n[0] + gb_n[1]) as f64 / gb_tot as f64,
        gb_hit_clas as f64 / gb_tot as f64,
        gb_fc_only as f64 / gb_tot as f64);
    println!("Home-to-1B times (n={}): mean {:.3}s ± {:.3}s | RHB (n={}) {:.3}s | LHB (n={}) {:.3}s | q10 {:.3} q50 {:.3} q90 {:.3}",
        h1b_all.len(), mean_sd(&h1b_all).0, mean_sd(&h1b_all).1,
        h1b_rhb.len(), mean_sd(&h1b_rhb).0,
        h1b_lhb.len(), mean_sd(&h1b_lhb).0,
        { h1b_all.sort_by(|a,b| a.total_cmp(b)); quantile(&h1b_all, 0.1) },
        quantile(&h1b_all, 0.5), quantile(&h1b_all, 0.9));

    println!("GB by LA sub-bin (<10 deg):");
    for (k, lbl) in ["<-15°", "-15..-5°", "-5..0°", "0..+5°", "+5..+10°"].iter().enumerate() {
        let tot = gb_la_n[k].max(1) as f64;
        println!("  {lbl:9} n {:4} ({:4.1}%) | Through {:4.1}% | IFH {:4.1}% | Out {:4.1}% | EV {:4.1} mph | bounces {:.2}",
            gb_la_n[k], 100.0 * gb_la_n[k] as f64 / gb_tot as f64,
            100.0 * gb_la_bins[k][0] as f64 / tot,
            100.0 * gb_la_bins[k][1] as f64 / tot,
            100.0 * gb_la_bins[k][2] as f64 / tot,
            gb_la_ev_sum[k] / tot, gb_la_bounces[k] as f64 / tot);
    }

    println!("GB Fielding Timelines:");
    let print_ts = |lbl: &str, ts: &GbTimeStats| {
        println!("  {lbl:10} n {:4} | pos[P,C,1B,2B,3B,SS] {:?} | bounces {:.2} | t_sec {:.2}s | t_g2r {:.2}s | throw_fly {:.2}s | dives att/ok/miss {}/{}/{} | stretch {} | no_throw {} late {} hur/off {}/{} | rim {} | jump wrong {}/{}",
            ts.n, ts.by_pos, mean_sd(&ts.bounces).0, mean_sd(&ts.t_sec).0, mean_sd(&ts.t_g2r).0, mean_sd(&ts.t_throw_fly).0,
            ts.dives_att, ts.dives_caught, ts.dives_missed, ts.stretch_delay, ts.no_throw, ts.threw_late, ts.hurried, ts.off_bal, ts.rim_deflect, ts.jump_wrong, ts.jump_moved);
    };
    print_ts("Through", &gb_ts[0]);
    print_ts("IFH", &gb_ts[1]);
    print_ts("Outs", &gb_ts[2]);
    println!("  Routine (<95mph, <=3m) n {} -> out {:.1}%:", routine_ts.n, 100.0 * routine_outs as f64 / routine_ts.n.max(1) as f64);
    print_ts("Routine", &routine_ts);
    println!("  Routine failures (outs=0, n={}): Through {routine_fail_thru}, IFH {routine_fail_ifh}, plays with IF dive {routine_fail_dive_miss}",
        routine_ts.n - routine_outs);
    ifh_dive_tsec.sort_by(|a, b| a.total_cmp(b));
    ifh_nodive_tsec.sort_by(|a, b| a.total_cmp(b));
    println!("  IFH split: Diving-caught (n={}) t_sec mean {:.2}s (p50 {:.2}, p90 {:.2}), g2r {:.2}s, no_throw {ifh_dive_nothrow} | Non-diving (n={}) t_sec mean {:.2}s (p50 {:.2}, p90 {:.2}), g2r {:.2}s, no_throw {ifh_nodive_nothrow} | rel_after_1B_touch {ifh_rel_after_1b}",
        ifh_dive_tsec.len(), mean_sd(&ifh_dive_tsec).0, quantile(&ifh_dive_tsec, 0.5), quantile(&ifh_dive_tsec, 0.9), mean_sd(&ifh_dive_tg2r).0,
        ifh_nodive_tsec.len(), mean_sd(&ifh_nodive_tsec).0, quantile(&ifh_nodive_tsec, 0.5), quantile(&ifh_nodive_tsec, 0.9), mean_sd(&ifh_nodive_tg2r).0);
    println!("  IFH rel<1B breakdown (n={ifh_rel_before_1b_n}): 1B_sec<=1b_but_touch_late_or_miss={ifh_1b_sec_before_touch} | 1B_sec_after_1b={ifh_1b_sec_after_1b} | no_1B_sec={ifh_1b_no_sec} | other_base={ifh_throw_other_base} | bounced={ifh_throw_bounced} | touch_late={ifh_1b_touch_late}");
    println!("  Through gap audit (n={thru_tot}): 2B-1B gap [+13..+38°]={thru_gap_2b1b} ({:.1}%), 3B-SS gap [-36..-13°]={thru_gap_3bss} ({:.1}%) | offline reach def={thru_reach_def} vs balanced(±32/±14.5°)={thru_reach_bal}",
        100.0 * thru_gap_2b1b as f64 / thru_tot.max(1) as f64,
        100.0 * thru_gap_3bss as f64 / thru_tot.max(1) as f64);

    println!("GB by 5-deg spray bin (All / Through% / IFH% / Out%):");
    for b in 0..18 {
        let lo = -45 + (b as i32) * 5;
        let hi = lo + 5;
        let tot = gb_spray_all[b].iter().sum::<u32>();
        let r_tot = gb_spray_rhb[b].iter().sum::<u32>();
        let l_tot = gb_spray_lhb[b].iter().sum::<u32>();
        println!("  {:3}..{:3}° n {:3} (R {:3}, L {:3}) | Thru {:4.1}% | IFH {:4.1}% | Out {:4.1}% | Hit% {:4.1}%",
            lo, hi, tot, r_tot, l_tot,
            100.0 * gb_spray_all[b][0] as f64 / tot.max(1) as f64,
            100.0 * gb_spray_all[b][1] as f64 / tot.max(1) as f64,
            100.0 * gb_spray_all[b][2] as f64 / tot.max(1) as f64,
            100.0 * (gb_spray_all[b][0] + gb_spray_all[b][1]) as f64 / tot.max(1) as f64);
    }

    println!("\nGB Through-the-hole & Hit% by Lane (ALL / RHB / LHB):");
    let lanes: &[(&str, usize, usize)] = &[
        ("3B line  [-45..-35°)", 0, 2),
        ("3B-SS    [-35..-15°)", 2, 6),
        ("SS-mid   [-15.. -5°)", 6, 8),
        ("P/mid    [ -5.. +5°)", 8, 10),
        ("mid-2B   [ +5..+15°)", 10, 12),
        ("2B-1B    [+15..+35°)", 12, 16),
        ("1B line  [+35..+45°]", 16, 18),
    ];
    for &(lbl, b0, b1) in lanes {
        let sum_lane = |arr: &[[u32; 3]; 18]| -> (u32, u32, u32, u32) {
            let mut s = [0u32; 3];
            for b in b0..b1 {
                for c in 0..3 { s[c] += arr[b][c]; }
            }
            (s[0] + s[1] + s[2], s[0], s[1], s[2])
        };
        let (an, at, ai, _) = sum_lane(&gb_spray_all);
        let (rn, rt, ri, _) = sum_lane(&gb_spray_rhb);
        let (ln, lt, li, _) = sum_lane(&gb_spray_lhb);
        println!(
            "  {lbl} | ALL n={an:4} Thru={at:3} ({:4.1}%) Hit={:4.1}% | RHB n={rn:4} Thru={rt:3} ({:4.1}%) Hit={:4.1}% | LHB n={ln:4} Thru={lt:3} ({:4.1}%) Hit={:4.1}%",
            100.0 * at as f64 / an.max(1) as f64,
            100.0 * (at + ai) as f64 / an.max(1) as f64,
            100.0 * rt as f64 / rn.max(1) as f64,
            100.0 * (rt + ri) as f64 / rn.max(1) as f64,
            100.0 * lt as f64 / ln.max(1) as f64,
            100.0 * (lt + li) as f64 / ln.max(1) as f64,
        );
    }

    println!("\nScorer Hit Types & Line Doubles (from PlayRuling):");
    for (lbl, s) in [("ALL", ld_all), ("RHB", ld_rhb), ("LHB", ld_lhb)] {
        println!(
            "  {lbl:3}: BIP {:4} | H {:4} ({:.3}) -> 1B {:4} 2B {:3} 3B {:2} HR {:3} | FC {:3} Out {:4} | 2B down 3B line: <=-35° {:2} (GB {:2}), <=-30° {:2} (GB {:2}) | down 1B line: >=+35° {:2} (GB {:2}), >=+30° {:2} (GB {:2})",
            s.bip, s.hits, s.hits as f64 / s.bip.max(1) as f64,
            s.singles, s.doubles, s.triples, s.hrs, s.fc, s.outs,
            s.doubles_3b_35, s.doubles_gb_3b_35, s.doubles_3b_30, s.doubles_gb_3b_30,
            s.doubles_1b_35, s.doubles_gb_1b_35, s.doubles_1b_30, s.doubles_gb_1b_30,
        );
    }
}

fn rel_idx_check(rel: &[(f64, u16)], fld: u16) -> Option<usize> {
    rel.iter().position(|(_, id)| *id == fld)
}
