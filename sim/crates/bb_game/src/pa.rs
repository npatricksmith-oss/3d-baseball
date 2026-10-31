//! Plate appearances without fielders (M3 cage, calibration-targets §5 B1).
//!
//! One pitch: the pitcher picks a type and target, releases with execution noise, the world
//! flies the ball, the batter perceives it through noisy observations and decides, the swing
//! is executed with noise, and the world resolves bat against ball. The pitch call comes
//! from geometry only (D1).

use bb_agents::batter::{self, Bank, Decision, Discipline, Perception, Scouting, eye_point, type_model};
use bb_agents::motor::{ReleaseNoise, SwingNoise, execute_release, execute_swing};
use bb_agents::pitcher::{PitchDef, PitchIntent, PitcherAttrs, plan_release};
use bb_agents::swing::{BatterAttrs, SwingTemplate, plan_swing};
use bb_math::{KeyedRng, Purpose, RngKey, Vec3, m};
use bb_observe::{BallObs, Count, PitchType, ReleaseObs, spin_signature};
use bb_physics::flight::{Crossing, DT_PITCH, fly};
use bb_physics::pitch::launch;
use bb_physics::spin::decompose;
use bb_physics::zone::{Y_JUDGE, Zone, ZoneRule};
use bb_physics::{BallState, FlightEnv};
use bb_world::pitch::{BatModel, SwingResult, resolve_pitch};

pub const PITCHER: u16 = 0x0100;
pub const BATTER: u16 = 0x0001;

/// One arsenal entry with usage by count group (ahead, even, behind, two strikes).
#[derive(Clone, Debug)]
pub struct ArsenalPitch {
    pub def: PitchDef,
    pub usage: [f64; 4],
}

pub fn standard_arsenal() -> Vec<ArsenalPitch> {
    let p = |ty, usage| ArsenalPitch { def: PitchDef::template(ty), usage };
    vec![
        p(PitchType::FourSeam, [0.48, 0.52, 0.62, 0.40]),
        p(PitchType::Slider, [0.25, 0.22, 0.16, 0.30]),
        p(PitchType::Changeup, [0.12, 0.13, 0.14, 0.12]),
        p(PitchType::Curveball, [0.15, 0.13, 0.08, 0.18]),
    ]
}

/// Global calibration knobs (PLAN §13.2). Each multiplies one family of noise scales.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Knobs {
    pub perception: f64,
    pub command: f64,
    pub swing: f64,
}

impl Default for Knobs {
    fn default() -> Self {
        Self { perception: 1.0, command: 1.0, swing: 1.0 }
    }
}

impl Knobs {
    /// Interim M3 calibration (docs/calibration/2026-10-31-m3-interim.md).
    pub fn calibrated_m3() -> Self {
        Self { perception: 2.5, command: 0.8, swing: 1.5 }
    }
}

/// Everything fixed for a batter–pitcher matchup.
pub struct Matchup {
    pub env: FlightEnv,
    pub pitcher: PitcherAttrs,
    pub arsenal: Vec<ArsenalPitch>,
    pub batter: BatterAttrs,
    pub perception: Perception,
    pub discipline: Discipline,
    pub template: SwingTemplate,
    pub bat: BatModel,
    pub zone: Zone,
    pub knobs: Knobs,
    /// Scouting with exact shapes (used when `model_error` is None).
    pub scouting: Scouting,
    pub model_error: Option<ModelError>,
}

/// Body capsule for hit-by-pitch (agent-models §3.7, v1 without evasion): a vertical
/// capsule on the batter's side of the plate.
const BODY_X: f64 = 0.80;
const BODY_R: f64 = 0.16;

impl Matchup {
    pub fn average() -> Self {
        let env = FlightEnv::default_env();
        let pitcher = PitcherAttrs::average_rhp();
        let batter = BatterAttrs::average_rhb();
        let arsenal = standard_arsenal();
        let zone = Zone::for_batter(batter.height, &ZoneRule::default());
        let scouting = build_scouting(&env, &pitcher, &arsenal, &zone);
        Matchup {
            env,
            pitcher,
            arsenal,
            batter,
            perception: Perception::default(),
            discipline: Discipline::default(),
            template: SwingTemplate::default(),
            bat: BatModel::default_34in(),
            zone,
            knobs: Knobs::default(),
            scouting,
            model_error: Some(ModelError::default()),
        }
    }

    /// Recompute what depends on the attributes (call after changing them).
    pub fn refresh(&mut self) {
        self.zone = Zone::for_batter(self.batter.height, &ZoneRule::default());
        self.scouting = build_scouting(&self.env, &self.pitcher, &self.arsenal, &self.zone);
    }
}

/// How wrong a batter's model of an unfamiliar pitcher's pitch shapes starts (agent-models
/// §3.8 σ_0) [E]. Adaptation (§3.8) shrinks it with exposure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelError {
    pub spin_axis_deg: f64,
    pub spin_rate_rel: f64,
    pub efficiency: f64,
    pub speed: f64,
}

impl Default for ModelError {
    fn default() -> Self {
        Self { spin_axis_deg: 10.0, spin_rate_rel: 0.08, efficiency: 0.08, speed: 0.6 }
    }
}

/// The batter's scouting models: each type's nominal path to the middle of the zone, built
/// from the batter's *believed* shape of each pitch (true shape plus model error).
pub fn build_scouting(env: &FlightEnv, p: &PitcherAttrs, arsenal: &[ArsenalPitch], zone: &Zone) -> Scouting {
    build_scouting_with(env, p, arsenal, zone, None)
}

pub fn build_scouting_with(env: &FlightEnv, p: &PitcherAttrs, arsenal: &[ArsenalPitch], zone: &Zone, err: Option<(ModelError, &mut KeyedRng)>) -> Scouting {
    let mid = 0.5 * (zone.bottom + zone.top);
    let mut types = Vec::new();
    let mut usage = Vec::new();
    let mut err = err;
    for a in arsenal {
        let mut plan = plan_release(env, p, &a.def, &PitchIntent { ty: a.def.ty, target_x: 0.0, target_z: mid }).expect("zone center reachable");
        if let Some((e, r)) = err.as_mut() {
            plan.shape.spin_axis_deg += e.spin_axis_deg * r.truncated_normal(4.0);
            plan.shape.spin_rate *= 1.0 + e.spin_rate_rel * r.truncated_normal(4.0);
            plan.shape.efficiency = (plan.shape.efficiency + e.efficiency * r.truncated_normal(4.0)).clamp(0.05, 1.0);
            plan.shape.speed += e.speed * r.truncated_normal(4.0);
            let a2 = bb_physics::pitch::aim(env, plan.r, &plan.shape, 0.0, mid, Some((plan.yaw, plan.elev)));
            plan.yaw = a2.yaw;
            plan.elev = a2.elev;
        }
        let s = launch(plan.r, &plan.shape, plan.yaw, plan.elev);
        let tr = fly(env, s, 0.0, DT_PITCH, &[Crossing::YBelow(-0.5)], 2.0);
        let d = decompose(s.w, s.v);
        let md = d.magnus_dir;
        let sig = spin_signature(md.x, md.z, d.efficiency, bb_math::units::radps_to_rpm(s.w.norm()));
        types.push(type_model(a.def.ty, tr, sig));
        usage.push(a.usage.iter().sum::<f64>() / 4.0);
    }
    Scouting { types, usage }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    Ball,
    CalledStrike,
    SwingingStrike,
    Foul,
    FoulTip,
    InPlay,
    HitByPitch,
}

#[derive(Clone, Debug)]
pub struct PitchRecord {
    pub ty: PitchType,
    pub count: Count,
    pub target: (f64, f64),
    pub plate: Option<Vec3>,
    pub in_zone: bool,
    pub swung: bool,
    pub check_swing: bool,
    pub contact: bool,
    pub call: Call,
    pub read: PitchType,
    pub read_weight: f64,
    pub decision: Option<Decision>,
    /// Batted-ball facts on contact: exit speed m/s, launch angle rad, spray rad.
    pub batted: Option<(f64, f64, f64)>,
    /// Final (adjusted) aim error at the swing's contact depth: (x, z) in m and timing in s
    /// (positive = aim later than the ball). Diagnostics only.
    pub aim_error: Option<Vec3>,
    /// Same, at commit (before adjustment).
    pub commit_error: Option<Vec3>,
}

fn rng(seed: u64, play: u32, agent: u16, purpose: Purpose, sub: u32) -> KeyedRng {
    KeyedRng::new(RngKey { game_seed: seed, play_index: play, agent_id: agent, purpose, sub_index: sub })
}

/// Bootstrap pitch selection (agent-models §2.4): usage by count group, then a target from
/// a count-based zone map. Sampling is a decision, not an outcome (D1).
pub fn choose_pitch(mu: &Matchup, count: Count, seed: u64, play: u32) -> PitchIntent {
    let mut r = rng(seed, play, PITCHER, Purpose::PitchSelection, 0);
    let g = batter::count_group(count);
    let tot: f64 = mu.arsenal.iter().map(|a| a.usage[g]).sum();
    let mut u = r.uniform() * tot;
    let mut ty = mu.arsenal[0].def.ty;
    for a in &mu.arsenal {
        if u < a.usage[g] {
            ty = a.def.ty;
            break;
        }
        u -= a.usage[g];
    }
    let z = &mu.zone;
    let mid = 0.5 * (z.bottom + z.top);
    let hh = 0.5 * (z.top - z.bottom);
    let p_zone = [0.42, 0.55, 0.72, 0.38][g];
    let breaking = matches!(ty, PitchType::Slider | PitchType::Curveball | PitchType::Sweeper | PitchType::Slurve | PitchType::KnuckleCurve | PitchType::Splitter | PitchType::Changeup);
    let (x, zt) = if r.uniform() < p_zone {
        // In the zone, toward the edges unless behind in the count.
        let spread = if g == 2 { 0.5 } else { 0.85 };
        let x = (2.0 * r.uniform() - 1.0) * spread * z.half_width;
        let zz = mid + (2.0 * r.uniform() - 1.0) * spread * hh - if breaking { 0.05 } else { 0.0 };
        (x, zz)
    } else if breaking {
        // Chase: just below the zone and/or off the glove-side corner (the shadow zone).
        let x = (2.0 * r.uniform() - 1.0) * z.half_width * 1.2;
        (x, z.bottom - 0.03 - 0.10 * r.uniform())
    } else {
        // Fastball chase: just above, or just off either edge.
        if r.uniform() < 0.5 {
            ((2.0 * r.uniform() - 1.0) * z.half_width, z.top + 0.03 + 0.07 * r.uniform())
        } else {
            let side = if r.uniform() < 0.5 { -1.0 } else { 1.0 };
            (side * (z.half_width + 0.04 + 0.07 * r.uniform()), mid + (2.0 * r.uniform() - 1.0) * hh)
        }
    };
    PitchIntent { ty, target_x: x, target_z: zt }
}

fn body_hit(mu: &Matchup, tr: &bb_physics::flight::Trajectory) -> bool {
    let side = match mu.batter.bats {
        bb_observe::Hand::Right => -1.0,
        bb_observe::Hand::Left => 1.0,
    };
    let bx = side * BODY_X;
    let (z0, z1) = (0.25, 0.80 * mu.batter.height);
    let reach = BODY_R + mu.env.ball.radius;
    tr.knots.iter().any(|k| {
        let r = k.s.r;
        if r.y < -0.1 || r.y > 0.7 {
            return false;
        }
        let dz = if r.z < z0 { z0 - r.z } else if r.z > z1 { r.z - z1 } else { 0.0 };
        let dx = r.x - bx;
        let dy = r.y - 0.25;
        dx * dx + dy * dy + dz * dz <= reach * reach
    })
}

/// Simulate one pitch at `count`, with the batter's scouting models `sc`.
pub fn simulate_pitch(mu: &Matchup, sc: &Scouting, count: Count, seed: u64, play: u32) -> PitchRecord {
    let intent = choose_pitch(mu, count, seed, play);
    let def = &mu.arsenal.iter().find(|a| a.def.ty == intent.ty).unwrap().def;
    let plan = plan_release(&mu.env, &mu.pitcher, def, &intent).expect("reachable target");
    let noise = ReleaseNoise::new(mu.pitcher.command, def.command_mult, mu.knobs.command);
    let release: BallState = execute_release(&plan, &noise, &mut rng(seed, play, PITCHER, Purpose::PitchRelease, 0)).state;

    // World: the pitch's own flight (independent of the swing until contact).
    let free = fly(&mu.env, release, 0.0, DT_PITCH, &[Crossing::YBelow(-1.2), Crossing::ZBelow(mu.env.ball.radius)], 3.0);
    let t_plate = free.find(Crossing::YBelow(Y_JUDGE));

    // Perception.
    let pc = &mu.perception;
    let kp = mu.knobs.perception;
    let eye = eye_point(mu.batter.bats, mu.batter.height);
    let mut pr = rng(seed, play, BATTER, Purpose::BatterPerception, 0);
    let rel_obs = ReleaseObs { t: 0.0, r: release.r + Vec3::new(0.05 * kp * pr.truncated_normal(4.0), 0.0, 0.05 * kp * pr.truncated_normal(4.0)) };
    let g = batter::count_group(count);
    let mut prior: Vec<f64> = mu.arsenal.iter().map(|a| a.usage[g]).collect();
    let ps: f64 = prior.iter().sum();
    prior.iter_mut().for_each(|p| *p /= ps);
    let mut bank = Bank::new(sc, &prior, &rel_obs, eye, pc.break_sigma);
    let d = decompose(release.w, release.v);
    let true_sig = spin_signature(d.magnus_dir.x, d.magnus_dir.z, d.efficiency, bb_math::units::radps_to_rpm(release.w.norm()));
    let spin_z = [pr.truncated_normal(4.0), pr.truncated_normal(4.0), pr.truncated_normal(4.0)];
    // Per-pitch persistent misjudgments (agent-models §3.3, M3): angle and time-to-arrival.
    let pers = [pr.truncated_normal(4.0), pr.truncated_normal(4.0), pr.truncated_normal(4.0)];
    let (wp, wi) = (m::sqrt(pc.persistent_share), m::sqrt(1.0 - pc.persistent_share));
    let t_cross_true = t_plate.unwrap_or(free.last().t);

    let observe = |bank: &mut Bank, i: u32| -> Option<f64> {
        let t = 0.010 * (i + 1) as f64;
        let (p, v) = free.at(t);
        if p.y - eye.y < pc.blank_dist || t >= free.last().t {
            return None;
        }
        let mut r = rng(seed, play, BATTER, Purpose::BatterPerception, i + 1);
        let (az, el) = batter::angles(eye, p);
        // Angular speed of the ball as seen from the eye.
        let d = p - eye;
        let omega = d.cross(v).norm() / d.norm_sq();
        let sa = pc.angular_sigma(omega) * kp;
        let tau = (t_cross_true - t).max(0.0);
        let st = pc.weber * kp * tau.max(0.02);
        let ss = pc.sigma_s0 * kp / m::sqrt(1.0 + t / pc.tau_s);
        let obs = BallObs {
            t,
            az: az + sa * (wp * pers[0] + wi * r.truncated_normal(4.0)),
            el: el + sa * (wp * pers[1] + wi * r.truncated_normal(4.0)),
            sigma_ang: sa,
            tau: tau + st * (wp * pers[2] + wi * r.truncated_normal(4.0)),
            sigma_tau: st,
            spin_cue: [true_sig[0] + ss * spin_z[0], true_sig[1] + ss * spin_z[1], true_sig[2] + ss * spin_z[2]],
            sigma_spin: ss,
        };
        bank.observe(sc, &obs);
        Some(t)
    };

    // Observe until commit.
    let mut i = 0u32;
    let mut t_now = 0.0;
    let decision;
    loop {
        let t_commit = bank.arrival() - mu.batter.t_swing - pc.latency;
        if t_now >= t_commit {
            break;
        }
        match observe(&mut bank, i) {
            Some(t) => t_now = t,
            None => break,
        }
        i += 1;
    }
    let post = bank.posterior();
    let aim = bank.dominant();
    let dec = batter::decide(&post, aim, count, (mu.zone.bottom, mu.zone.top), &mu.discipline, 0.0, mu.env.ball.radius);
    decision = Some(dec);

    let mut swung = dec.swing;
    let mut check = false;
    let mut swing_res = SwingResult::NoSwing;
    let mut resolved_pitch = free.clone();
    let mut aim_error = None;
    let mut commit_error = None;
    let truth_err = |p: Vec3, t: f64| -> Option<Vec3> {
        let tt = free.find(Crossing::YBelow(p.y))?;
        let q = free.at(tt).0;
        Some(Vec3::new(p.x - q.x, p.z - q.z, t - tt))
    };
    if swung {
        let aim_pt = bank.point_at_depth(sc, aim.ty, mu.template.contact_y(&mu.batter, aim.mean.x));
        // Keep observing to the cutoff; adjust within limits; maybe check the swing.
        let t_commit = t_now;
        commit_error = aim_pt.and_then(|(p, t)| truth_err(p, t));
        let t_aim0 = aim_pt.map(|a| a.1).unwrap_or(bank.arrival());
        let t_cut = t_aim0 - 0.060;
        let t_abort = t_aim0 - 0.55 * mu.batter.t_swing;
        while t_now < t_cut {
            match observe(&mut bank, i) {
                Some(t) => t_now = t,
                None => break,
            }
            i += 1;
            if t_now < t_abort {
                let post2 = bank.posterior();
                let d2 = batter::decide(&post2, bank.dominant(), count, (mu.zone.bottom, mu.zone.top), &mu.discipline, 0.0, mu.env.ball.radius);
                if d2.ev_swing - d2.ev_take < -0.03 {
                    check = true;
                    swung = false;
                    break;
                }
            }
        }
        if !check {
            let late = bank.dominant();
            let late_pt = bank.point_at_depth(sc, late.ty, mu.template.contact_y(&mu.batter, late.mean.x));
            if let (Some((p0, t0)), Some((p1, t1))) = (aim_pt, late_pt) {
                // Bounded adjustment (agent-models §3.6): v_adj on position, ±10% on timing.
                let v_adj = 0.6;
                let span = (t_now - t_commit).max(0.0);
                let lim = v_adj * span;
                let dp = p1 - p0;
                let dp = Vec3::new(dp.x.clamp(-lim, lim), dp.y, dp.z.clamp(-lim, lim));
                let dt_lim = 0.10 * (t0 - t_commit).max(0.0);
                let t_aim = t0 + (t1 - t0).clamp(-dt_lim, dt_lim);
                let aim3 = p0 + dp + Vec3::new(0.0, 0.0, mu.template.aim_offset_z);
                aim_error = truth_err(p0 + dp, t_aim);
                let intended = plan_swing(&mu.batter, &mu.template, &mu.bat.profile, &mu.bat.mass, aim3, t_aim, mu.template.attack_angle);
                let sn = SwingNoise::new(&mu.batter, mu.knobs.swing);
                let ex = execute_swing(&intended, &sn, &mut rng(seed, play, BATTER, Purpose::SwingExecution, 0));
                let res = resolve_pitch(&mu.env, release, Some(&ex.swing), &mu.bat);
                resolved_pitch = res.pitch;
                swing_res = res.swing;
            } else {
                swing_res = SwingResult::Miss(bb_physics::contact::ClosestApproach { t: t_aim0, u: 0.0, gap: f64::INFINITY, offset: Vec3::ZERO });
            }
        }
    }

    let plate = t_plate.map(|t| free.at(t).0);
    let bounced = t_plate.is_none() || free.stop == Some(1) && free.last().t < t_plate.unwrap_or(f64::INFINITY);
    let in_zone = match plate {
        Some(p) if !bounced => mu.zone.is_strike(p.x, p.z, mu.env.ball.radius),
        _ => false,
    };
    let (call, batted, contact) = match &swing_res {
        SwingResult::Contact(b) => {
            let v_in = resolved_pitch.last().s.v.normalized();
            let v_out = b.contact.ball_after.v;
            let cosang = v_out.normalized().dot(v_in);
            let land = b.flight.last().s.r;
            let fair = land.y > 0.0 && land.x.abs() <= land.y;
            let call = if cosang > m::cos(25.0f64.to_radians()) && v_out.norm() > 0.5 * resolved_pitch.last().s.v.norm() {
                Call::FoulTip
            } else if fair {
                Call::InPlay
            } else {
                Call::Foul
            };
            (call, Some((b.exit_speed, b.launch_angle, b.spray_angle)), true)
        }
        SwingResult::Miss(_) => (Call::SwingingStrike, None, false),
        SwingResult::NoSwing => {
            if body_hit(mu, &free) {
                (Call::HitByPitch, None, false)
            } else if in_zone {
                (Call::CalledStrike, None, false)
            } else {
                (Call::Ball, None, false)
            }
        }
    };
    let read = aim;
    PitchRecord {
        ty: intent.ty,
        count,
        target: (intent.target_x, intent.target_z),
        plate,
        in_zone,
        swung,
        check_swing: check,
        contact,
        call,
        read: read.ty,
        read_weight: read.w,
        decision,
        batted,
        aim_error,
        commit_error,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaResult {
    Strikeout,
    Walk,
    HitByPitch,
    InPlay,
}

#[derive(Clone, Debug)]
pub struct PaRecord {
    pub pitches: Vec<PitchRecord>,
    pub result: PaResult,
    pub called_third: bool,
}

/// The batter's scouting for one plate appearance: exact, or with fresh model error drawn
/// from the keyed RNG (the cage treats each PA as facing an unfamiliar pitcher).
pub fn scouting_for_pa(mu: &Matchup, seed: u64, first_play: u32) -> std::borrow::Cow<'_, Scouting> {
    match mu.model_error {
        None => std::borrow::Cow::Borrowed(&mu.scouting),
        Some(e) => {
            let mut r = rng(seed, first_play, BATTER, Purpose::BatterPerception, 1_000_000);
            std::borrow::Cow::Owned(build_scouting_with(&mu.env, &mu.pitcher, &mu.arsenal, &mu.zone, Some((e, &mut r))))
        }
    }
}

/// Simulate a plate appearance. Play indices `first_play..` are used for the pitches.
pub fn simulate_pa(mu: &Matchup, seed: u64, first_play: u32) -> PaRecord {
    let sc = scouting_for_pa(mu, seed, first_play);
    let sc: &Scouting = &sc;
    let mut count = Count::default();
    let mut pitches = Vec::new();
    let mut play = first_play;
    loop {
        let rec = simulate_pitch(mu, sc, count, seed, play);
        play += 1;
        let call = rec.call;
        pitches.push(rec);
        match call {
            Call::Ball => {
                count.balls += 1;
                if count.balls == 4 {
                    return PaRecord { pitches, result: PaResult::Walk, called_third: false };
                }
            }
            Call::CalledStrike | Call::SwingingStrike | Call::FoulTip => {
                count.strikes += 1;
                if count.strikes == 3 {
                    return PaRecord { pitches, result: PaResult::Strikeout, called_third: call == Call::CalledStrike };
                }
            }
            Call::Foul => {
                if count.strikes < 2 {
                    count.strikes += 1;
                }
            }
            Call::HitByPitch => return PaRecord { pitches, result: PaResult::HitByPitch, called_third: false },
            Call::InPlay => return PaRecord { pitches, result: PaResult::InPlay, called_third: false },
        }
        if pitches.len() > 40 {
            // Liveness guard (rules-spec): an endless foul streak is a model bug; end it.
            return PaRecord { pitches, result: PaResult::InPlay, called_third: false };
        }
    }
}

/// Plate-discipline and batted-ball rates (calibration-targets §4.4, §5).
#[derive(Clone, Debug, Default)]
pub struct B1Report {
    pub pa: u32,
    pub k: u32,
    pub bb: u32,
    pub hbp: u32,
    pub bip: u32,
    pub pitches: u32,
    pub zone: u32,
    pub first_pitch_strikes: u32,
    pub swings: u32,
    pub z_swings: u32,
    pub o_swings: u32,
    pub contacts: u32,
    pub z_contacts: u32,
    pub o_contacts: u32,
    pub whiffs: u32,
    pub fouls: u32,
    pub called_k: u32,
    pub checks: u32,
    pub bbe_ev: Vec<f64>,
    pub bbe_la: Vec<f64>,
}

impl B1Report {
    pub fn add(&mut self, pa: &PaRecord) {
        self.pa += 1;
        match pa.result {
            PaResult::Strikeout => {
                self.k += 1;
                self.called_k += pa.called_third as u32;
            }
            PaResult::Walk => self.bb += 1,
            PaResult::HitByPitch => self.hbp += 1,
            PaResult::InPlay => self.bip += 1,
        }
        if let Some(f) = pa.pitches.first() {
            self.first_pitch_strikes += matches!(f.call, Call::CalledStrike | Call::SwingingStrike | Call::Foul | Call::FoulTip | Call::InPlay) as u32;
        }
        for p in &pa.pitches {
            self.pitches += 1;
            self.zone += p.in_zone as u32;
            self.checks += p.check_swing as u32;
            if p.swung {
                self.swings += 1;
                if p.in_zone { self.z_swings += 1 } else { self.o_swings += 1 }
                if p.contact {
                    self.contacts += 1;
                    if p.in_zone { self.z_contacts += 1 } else { self.o_contacts += 1 }
                } else {
                    self.whiffs += 1;
                }
            }
            if matches!(p.call, Call::Foul | Call::FoulTip) {
                self.fouls += 1;
            }
            if let Some((ev, la, _)) = p.batted {
                self.bbe_ev.push(bb_math::units::mps_to_mph(ev));
                self.bbe_la.push(la.to_degrees());
            }
        }
    }

    fn pct(a: u32, b: u32) -> f64 {
        100.0 * a as f64 / b.max(1) as f64
    }

    /// (id, metric, value, target, envelope low, envelope high)
    pub fn rows(&self) -> Vec<(&'static str, &'static str, f64, f64, f64, f64)> {
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
        let sd = |v: &[f64]| {
            let mu = mean(v);
            (v.iter().map(|x| (x - mu) * (x - mu)).sum::<f64>() / v.len().max(1) as f64).sqrt()
        };
        let hard = self.bbe_ev.iter().filter(|&&e| e >= 95.0).count() as u32;
        let n = self.bbe_ev.len() as u32;
        vec![
            ("B1.1", "K%", Self::pct(self.k, self.pa), 22.5, 15.0, 28.0),
            ("B1.2", "BB%", Self::pct(self.bb, self.pa), 8.3, 5.0, 12.0),
            ("B1.3", "HBP%", Self::pct(self.hbp, self.pa), 1.1, 0.4, 2.0),
            ("B1.4", "BIP%", Self::pct(self.bip, self.pa), 68.0, 60.0, 100.0),
            ("B1.5", "Pitches/PA", self.pitches as f64 / self.pa.max(1) as f64, 3.9, 3.5, 4.3),
            ("B1.6", "Zone%", Self::pct(self.zone, self.pitches), 48.0, 40.0, 55.0),
            ("B1.7", "F-Strike%", Self::pct(self.first_pitch_strikes, self.pa), 61.0, 54.0, 68.0),
            ("B1.8", "Swing%", Self::pct(self.swings, self.pitches), 47.0, 40.0, 54.0),
            ("B1.9", "Z-Swing%", Self::pct(self.z_swings, self.zone), 67.0, 58.0, 76.0),
            ("B1.10", "O-Swing%", Self::pct(self.o_swings, self.pitches - self.zone), 29.0, 20.0, 36.0),
            ("B1.11", "Contact%", Self::pct(self.contacts, self.swings), 76.0, 70.0, 83.0),
            ("B1.12", "Z-Contact%", Self::pct(self.z_contacts, self.z_swings), 85.0, 79.0, 91.0),
            ("B1.13", "O-Contact%", Self::pct(self.o_contacts, self.o_swings), 57.0, 45.0, 68.0),
            ("B1.14", "SwStr%", Self::pct(self.whiffs, self.pitches), 11.0, 8.0, 15.0),
            ("B1.15", "Foul/contact%", Self::pct(self.fouls, self.contacts), 50.0, 40.0, 60.0),
            ("B1.16", "Called-K share%", Self::pct(self.called_k, self.k), 25.0, 12.0, 38.0),
            ("4.4", "Mean EV (mph)", mean(&self.bbe_ev), 88.5, 85.0, 92.0),
            ("4.4", "Mean LA (deg)", mean(&self.bbe_la), 12.0, 6.0, 18.0),
            ("4.4", "LA SD (deg)", sd(&self.bbe_la), 27.0, 20.0, 35.0),
            ("4.4", "Hard-hit%", Self::pct(hard, n), 38.0, 30.0, 47.0),
        ]
    }

    pub fn print(&self) {
        println!("{:6} {:16} {:>8} {:>8}  {:>13}", "id", "metric", "sim", "target", "envelope");
        for (id, name, v, t, lo, hi) in self.rows() {
            let flag = if v < lo || v > hi { "  <-- OUT" } else { "" };
            println!("{id:6} {name:16} {v:8.1} {t:8.1}  {lo:6.1}–{hi:<6.1}{flag}");
        }
        println!("check swings: {}", self.checks);
    }
}

/// Run `n` plate appearances and collect the B1 report.
pub fn run_pas(mu: &Matchup, n: u32, seed: u64) -> B1Report {
    let mut rep = B1Report::default();
    for i in 0..n {
        rep.add(&simulate_pa(mu, seed, i * 64));
    }
    rep
}

impl Matchup {
    /// Average matchup with the interim M3 knobs, and the given grades applied.
    pub fn with_grades(b: &bb_agents::attributes::BatterGrades, p: &bb_agents::attributes::PitcherGrades) -> Matchup {
        let mut mu = Matchup::average();
        mu.knobs = Knobs::calibrated_m3();
        b.apply(&mut mu.batter, &mut mu.perception, &mut mu.discipline);
        p.apply(&mut mu.pitcher);
        mu.refresh();
        mu
    }
}
