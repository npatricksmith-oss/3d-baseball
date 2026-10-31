//! Plate appearances without fielders (M3 cage, calibration-targets §5 B1).
//!
//! One pitch: the pitcher picks a type and target, releases with execution noise, the world
//! flies the ball, the batter perceives it through noisy observations and decides, the swing
//! is executed with noise, and the world resolves bat against ball. The pitch call comes
//! from geometry only (D1).

use bb_agents::batter::{self, Bank, Decision, Discipline, OwnExecution, Perception, PitcherMemory, Scouting, eye_point, type_model};
use bb_agents::motor::{ReleaseNoise, SwingNoise, execute_release, execute_swing};
use bb_agents::pitcher::{PitchDef, PitchIntent, PitcherAttrs, plan_release, plan_release_seeded};
use bb_agents::swing::{BatterAttrs, SwingTemplate, apply_placement, plan_swing};
use bb_math::{KeyedRng, Purpose, RngKey, Vec3, m};
use bb_observe::{BallObs, Count, PitchType, ReleaseObs, spin_signature};
use bb_physics::flight::{Crossing, DT_PITCH, fly};
use bb_physics::pitch::launch;
use bb_physics::spin::decompose;
use bb_physics::zone::{Y_JUDGE, Zone, ZoneRule};
use bb_physics::{BallState, FlightEnv};
use bb_world::pitch::{BatModel, SwingResult, pitch_flight, resolve_flight};

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
        Self { perception: 2.25, command: 0.8, swing: 1.0 }
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
    /// The pitcher's cached aim per arsenal pitch: (yaw, elev, target x, target z, distance).
    pub aim_seeds: Vec<(f64, f64, f64, f64, f64)>,
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
            aim_seeds: Vec::new(),
            scouting,
            model_error: Some(ModelError::default()),
        }
        .seeded()
    }

    /// Recompute what depends on the attributes (call after changing them).
    pub fn refresh(&mut self) {
        self.zone = Zone::for_batter(self.batter.height, &ZoneRule::default());
        self.scouting = build_scouting(&self.env, &self.pitcher, &self.arsenal, &self.zone);
        self.aim_seeds = aim_seeds(&self.env, &self.pitcher, &self.arsenal, &self.zone);
    }

    fn seeded(mut self) -> Self {
        self.aim_seeds = aim_seeds(&self.env, &self.pitcher, &self.arsenal, &self.zone);
        self
    }

    /// Initial guess for aiming `intent`, from the cached solve for its type.
    pub fn aim_guess(&self, intent: &PitchIntent) -> Option<(f64, f64)> {
        let k = self.arsenal.iter().position(|a| a.def.ty == intent.ty)?;
        let &(yaw, elev, x, z, d) = self.aim_seeds.get(k)?;
        Some((yaw + (intent.target_x - x) / d, elev + (intent.target_z - z) / d))
    }
}

/// The pitcher's game-start aim solves to the middle of the zone, one per arsenal pitch.
pub fn aim_seeds(env: &FlightEnv, p: &PitcherAttrs, arsenal: &[ArsenalPitch], zone: &Zone) -> Vec<(f64, f64, f64, f64, f64)> {
    let mid = 0.5 * (zone.bottom + zone.top);
    arsenal
        .iter()
        .map(|a| {
            let plan = plan_release(env, p, &a.def, &PitchIntent { ty: a.def.ty, target_x: 0.0, target_z: mid }).expect("zone center reachable");
            let d = (Vec3::new(0.0, Y_JUDGE, mid) - plan.r).norm();
            (plan.yaw, plan.elev, 0.0, mid, d)
        })
        .collect()
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
        Self { spin_axis_deg: 15.0, spin_rate_rel: 0.08, efficiency: 0.08, speed: 0.6 }
    }
}

/// The batter's scouting models: each type's nominal path to the middle of the zone, built
/// from the batter's *believed* shape of each pitch (true shape plus model error).
pub fn build_scouting(env: &FlightEnv, p: &PitcherAttrs, arsenal: &[ArsenalPitch], zone: &Zone) -> Scouting {
    build_scouting_with(env, p, arsenal, zone, None)
}

/// Draw a batter's initial model error for each arsenal pitch (σ_0 of agent-models §3.8).
pub fn draw_offsets(arsenal: &[ArsenalPitch], e: &ModelError, r: &mut KeyedRng) -> Vec<[f64; 4]> {
    arsenal
        .iter()
        .map(|_| {
            [
                e.spin_axis_deg * r.truncated_normal(4.0),
                e.spin_rate_rel * r.truncated_normal(4.0),
                e.efficiency * r.truncated_normal(4.0),
                e.speed * r.truncated_normal(4.0),
            ]
        })
        .collect()
}

/// Scouting models built from believed shapes = true shape + per-type offsets.
pub fn build_scouting_offsets(env: &FlightEnv, p: &PitcherAttrs, arsenal: &[ArsenalPitch], zone: &Zone, offsets: &[[f64; 4]]) -> Scouting {
    let mid = 0.5 * (zone.bottom + zone.top);
    let mut types = Vec::new();
    let mut usage = Vec::new();
    for (a, o) in arsenal.iter().zip(offsets) {
        let mut plan = plan_release(env, p, &a.def, &PitchIntent { ty: a.def.ty, target_x: 0.0, target_z: mid }).expect("zone center reachable");
        plan.shape.spin_axis_deg += o[0];
        plan.shape.spin_rate *= 1.0 + o[1];
        plan.shape.efficiency = (plan.shape.efficiency + o[2]).clamp(0.05, 1.0);
        plan.shape.speed += o[3];
        let a2 = bb_physics::pitch::aim(env, plan.r, &plan.shape, 0.0, mid, Some((plan.yaw, plan.elev)));
        let s = launch(plan.r, &plan.shape, a2.yaw, a2.elev);
        let tr = fly(env, s, 0.0, DT_PITCH, &[Crossing::YBelow(-0.5)], 2.0);
        let d = decompose(s.w, s.v);
        let md = d.magnus_dir;
        let sig = spin_signature(md.x, md.z, d.efficiency, bb_math::units::radps_to_rpm(s.w.norm()));
        types.push(type_model(a.def.ty, tr, sig));
        usage.push(a.usage.iter().sum::<f64>() / 4.0);
    }
    Scouting { types, usage }
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
    let bs = vec![mu.perception.break_sigma; sc.types.len()];
    simulate_pitch_ctx(mu, sc, count, seed, play, &bs, None)
}

/// [`simulate_pitch`] with per-type break-spread beliefs (from in-game adaptation) and the
/// previous pitch type seen in this plate appearance (sequence prior, agent-models §3.1).
pub fn simulate_pitch_ctx(mu: &Matchup, sc: &Scouting, count: Count, seed: u64, play: u32, break_sigma: &[f64], prev_read: Option<PitchType>) -> PitchRecord {
    let intent = choose_pitch(mu, count, seed, play);
    let def = &mu.arsenal.iter().find(|a| a.def.ty == intent.ty).unwrap().def;
    let plan = plan_release_seeded(&mu.env, &mu.pitcher, def, &intent, mu.aim_guess(&intent)).expect("reachable target");
    let noise = ReleaseNoise::new(mu.pitcher.command, def.command_mult, mu.knobs.command);
    let release: BallState = execute_release(&plan, &noise, &mut rng(seed, play, PITCHER, Purpose::PitchRelease, 0)).state;

    // World: the pitch's own flight (independent of the swing until contact).
    let free = pitch_flight(&mu.env, release);
    let t_plate = free.find(Crossing::YBelow(Y_JUDGE));

    // ---- Perception (agent-models §3.2–§3.3). Observations are built here from the true
    // flight plus noise; the batter only ever receives the noisy values.
    let pc = &mu.perception;
    let kp = mu.knobs.perception;
    let eye = eye_point(mu.batter.bats, mu.batter.height);
    let mut pr = rng(seed, play, BATTER, Purpose::BatterPerception, 0);
    let rel_sigma = batter::RELEASE_SIGMA * kp;
    let rel_obs = ReleaseObs {
        t: 0.0,
        r: release.r + Vec3::new(rel_sigma * pr.truncated_normal(4.0), 0.0, rel_sigma * pr.truncated_normal(4.0)),
        sigma: rel_sigma,
    };
    // Prior over types: usage by count, then the sequence factor (repeats slightly less
    // expected, agent-models §3.1).
    let g = batter::count_group(count);
    let mut prior: Vec<f64> = mu
        .arsenal
        .iter()
        .map(|a| a.usage[g] * if Some(a.def.ty) == prev_read { SEQ_REPEAT } else { 1.0 })
        .collect();
    let ps: f64 = prior.iter().sum();
    prior.iter_mut().for_each(|p| *p /= ps);
    let (wp, wi) = (m::sqrt(pc.persistent_share), m::sqrt(1.0 - pc.persistent_share));
    let bias = batter::BiasPrior { ang: pc.sigma_ang * kp * wp, tau_rel: pc.weber * kp * wp };
    let mut bank = Bank::new(sc, &prior, &rel_obs, eye, break_sigma, bias);
    let d = decompose(release.w, release.v);
    let true_sig = spin_signature(d.magnus_dir.x, d.magnus_dir.z, d.efficiency, bb_math::units::radps_to_rpm(release.w.norm()));
    let spin_z = [pr.truncated_normal(4.0), pr.truncated_normal(4.0), pr.truncated_normal(4.0)];
    // This pitch's persistent misjudgments (the bias states' true values).
    let pers = [bias.ang * pr.truncated_normal(4.0), bias.ang * pr.truncated_normal(4.0), bias.tau_rel * pr.truncated_normal(4.0)];
    let t_cross_true = t_plate.unwrap_or(free.last().t);

    let observe = |bank: &mut Bank, i: u32| -> Option<f64> {
        let t = 0.010 * (i + 1) as f64;
        let (p, v) = free.at(t);
        if p.y - eye.y < pc.blank_dist || t >= free.last().t {
            return None;
        }
        let mut r = rng(seed, play, BATTER, Purpose::BatterPerception, i + 1);
        let (az, el) = batter::angles(eye, p);
        let dv = p - eye;
        let omega = dv.cross(v).norm() / dv.norm_sq();
        let sa = pc.angular_sigma(omega) * kp * wi;
        let tau = (t_cross_true - t).max(0.0);
        let tau_obs = (tau * (1.0 + pers[2] + pc.weber * kp * wi * r.truncated_normal(4.0))).max(0.0);
        let ss = pc.sigma_s0 * kp / m::sqrt(1.0 + t / pc.tau_s);
        let obs = BallObs {
            t,
            az: az + pers[0] + sa * r.truncated_normal(4.0),
            el: el + pers[1] + sa * r.truncated_normal(4.0),
            sigma_ang: sa,
            tau: tau_obs,
            // From the perceived τ only (Codex M3 review #2).
            sigma_tau: pc.weber * kp * wi * tau_obs.max(0.02),
            spin_cue: [true_sig[0] + ss * spin_z[0], true_sig[1] + ss * spin_z[1], true_sig[2] + ss * spin_z[2]],
            sigma_spin: ss,
        };
        bank.observe(sc, &obs);
        Some(t)
    };

    // ---- Observe until commit.
    let mut i = 0u32;
    let mut t_now = 0.0;
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
    let zone = (mu.zone.bottom, mu.zone.top);
    let post = bank.posterior();
    let aim = bank.dominant();
    // Expected correction between now and the cutoff (60 ms before the believed arrival).
    let t_left = (bank.t_release + aim.mean.z - 0.060 - t_now).max(0.0);
    let own = OwnExecution {
        vertical: mu.batter.bat_control * mu.knobs.swing,
        timing: mu.batter.timing * mu.knobs.swing,
        correct_z: batter::CORRECTION_CREDIT * mu.batter.adjustability * t_left,
        correct_t: batter::CORRECTION_CREDIT * 0.10 * (t_left + 0.060 - pc.latency).max(0.0),
    };
    let dec = batter::decide(&post, aim, count, zone, &mu.discipline, own, 0.0, mu.env.ball.radius);

    // ---- Swing: planned and executed at commit, then adjusted causally (agent-models §3.6).
    let mut swing_res = SwingResult::NoSwing;
    let mut resolved_pitch = free.clone();
    let (mut aim_error, mut commit_error) = (None, None);
    let truth_err = |p: Vec3, t: f64| -> Option<Vec3> {
        let tt = free.find(Crossing::YBelow(p.y))?;
        let q = free.at(tt).0;
        Some(Vec3::new(p.x - q.x, p.z - q.z, t - tt))
    };
    let mut swung = dec.swing;
    let mut check = false;
    if dec.swing {
        let depth = |x: f64| mu.template.contact_y(&mu.batter, x);
        // Where to aim: the dominant hypothesis' path at contact depth; if that prediction is
        // unavailable, its plate-plane mean (Codex M3 review #8).
        let aim_of = |bank: &Bank, c: &batter::Component| -> (Vec3, f64) {
            bank.point_at_depth(sc, c.ty, depth(c.mean.x))
                .unwrap_or((Vec3::new(c.mean.x, Y_JUDGE, c.mean.y), bank.t_release + c.mean.z))
        };
        let (mut cur, mut t_aim) = aim_of(&bank, &aim);
        commit_error = truth_err(cur, t_aim);
        let off = Vec3::new(0.0, 0.0, mu.template.aim_offset_z);
        let planner = |p: Vec3, t: f64| plan_swing(&mu.batter, &mu.template, &mu.bat.profile, &mu.bat.mass, mu.bat.contact.ss_from_tip, p + off, t, mu.template.attack_angle);
        let sn = SwingNoise::new(&mu.batter, mu.knobs.swing);
        let ex = execute_swing(&planner(cur, t_aim), &sn, &mut rng(seed, play, BATTER, Purpose::SwingExecution, 0));
        let mut swing = ex.swing;
        let t_commit = t_now;
        // Phase rate may change ±10% over the swing still to come after motor latency.
        let dt_lim = 0.10 * (t_aim - (t_commit + pc.latency)).max(0.0);
        let t_aim0 = t_aim;
        let t_abort = swing.t_start() + 0.6 * (swing.t_c - swing.t_start());
        let v_adj = mu.batter.adjustability;
        let mut t_last = t_commit;
        loop {
            let t_cut = t_aim - 0.060;
            if t_now + 0.010 > t_cut {
                break;
            }
            match observe(&mut bank, i) {
                Some(t) => t_now = t,
                None => break,
            }
            i += 1;
            let post2 = bank.posterior();
            let dom = bank.dominant();
            if t_now < t_abort {
                let d2 = batter::decide(&post2, dom, count, zone, &mu.discipline, own, 0.0, mu.env.ball.radius);
                if d2.ev_swing - d2.ev_take < -0.03 {
                    // Brake after motor latency; the rules judge the result (rules-spec §4.5).
                    swing.t_brake = Some(t_now + pc.latency);
                    check = true;
                    break;
                }
            }
            // Bounded move of the aim toward the new estimate since the last tick.
            let (want, t_want) = aim_of(&bank, &dom);
            let lim = v_adj * (t_now - t_last);
            t_last = t_now;
            let dx = (want.x - cur.x).clamp(-lim, lim);
            let dz = (want.z - cur.z).clamp(-lim, lim);
            let nx = cur.x + dx;
            cur = Vec3::new(nx, depth(nx), cur.z + dz);
            t_aim = t_aim0 + (t_want - t_aim0).clamp(-dt_lim, dt_lim);
            // The same executed placement deviation rides on the adjusted plan.
            let mut s2 = apply_placement(&planner(cur, t_aim), &ex.placement);
            s2.t_brake = swing.t_brake;
            swing = s2;
        }
        aim_error = truth_err(cur, t_aim);
        let res = resolve_flight(&mu.env, free.clone(), Some(&swing), &mu.bat);
        resolved_pitch = res.pitch;
        swing_res = res.swing;
        // Any contact is a swing; otherwise a braked swing is judged by its yaw.
        let contact = matches!(swing_res, SwingResult::Contact(_));
        swung = contact || swing.judged_swing();
    }

    let plate = t_plate.map(|t| free.at(t).0);
    let bounced = t_plate.is_none() || free.stop == Some(1) && free.last().t < t_plate.unwrap_or(f64::INFINITY);
    let in_zone = match plate {
        Some(p) if !bounced => mu.zone.is_strike(p.x, p.z, mu.env.ball.radius),
        _ => false,
    };
    let (call, batted, contact) = match &swing_res {
        SwingResult::Contact(b) => {
            let land = b.flight.last().s.r;
            let fair = land.y > 0.0 && land.x.abs() <= land.y;
            // No catcher exists yet, so no contact can be a caught foul tip (rules-spec §4.4):
            // backward contact is a foul.
            let call = if fair { Call::InPlay } else { Call::Foul };
            (call, Some((b.exit_speed, b.launch_angle, b.spray_angle)), true)
        }
        _ if swung => (Call::SwingingStrike, None, false),
        _ => {
            if body_hit(mu, &free) {
                (Call::HitByPitch, None, false)
            } else if in_zone {
                (Call::CalledStrike, None, false)
            } else {
                (Call::Ball, None, false)
            }
        }
    };
    let _ = &resolved_pitch;
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
        read: aim.ty,
        read_weight: aim.w,
        decision: Some(dec),
        batted,
        aim_error,
        commit_error,
    }
}

/// Factor on the prior of the type just seen in this PA (agent-models §3.1) [E].
pub const SEQ_REPEAT: f64 = 0.85;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaResult {
    Strikeout,
    Walk,
    HitByPitch,
    InPlay,
    /// The liveness guard stopped an endless plate appearance: a model bug, not a result.
    Unresolved,
}

/// Liveness guard for one plate appearance; play-index spacing between PAs must exceed it.
pub const MAX_PA_PITCHES: usize = 200;
/// Play-index stride between plate appearances in batch runs.
pub const PA_PLAY_STRIDE: u32 = 1000;

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
    let bs = vec![mu.perception.break_sigma; sc.types.len()];
    loop {
        let prev = pitches.last().map(|p: &PitchRecord| p.read);
        let rec = simulate_pitch_ctx(mu, sc, count, seed, play, &bs, prev);
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
        if pitches.len() >= MAX_PA_PITCHES {
            // Liveness guard: report it, never substitute a baseball result (D1).
            return PaRecord { pitches, result: PaResult::Unresolved, called_third: false };
        }
    }
}

/// Plate-discipline and batted-ball rates (calibration-targets §4.4, §5).
#[derive(Clone, Debug, Default)]
pub struct B1Report {
    pub pa: u32,
    pub unresolved: u32,
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
            PaResult::Unresolved => self.unresolved += 1,
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
        println!("check swings: {}; unresolved PAs (liveness): {}", self.checks, self.unresolved);
    }
}

/// Run `n` plate appearances and collect the B1 report.
pub fn run_pas(mu: &Matchup, n: u32, seed: u64) -> B1Report {
    let mut rep = B1Report::default();
    for i in 0..n {
        rep.add(&simulate_pa(mu, seed, i * PA_PLAY_STRIDE));
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

/// Adaptation rate η from `Adaptation` (agent-models §10, grade 50).
pub const ETA: f64 = 0.15;
/// On-deck viewing weight (agent-models §3.8).
pub const ON_DECK_WEIGHT: f64 = 0.3;
/// Perceptual noise of one sighting when learning shapes (same units as the offsets) [E].
const CUE_NOISE: [f64; 4] = [6.0, 0.05, 0.05, 0.4];

/// One batter's sequence of plate appearances against the same pitcher in one game, with
/// in-game adaptation (agent-models §3.8). Between PAs the batter watches `on_deck_pitches`
/// pitches to other hitters at reduced weight. Returns the PA records in order.
pub fn simulate_times_through(mu: &Matchup, seed: u64, game: u32, times: u32, on_deck_pitches: u32, adapt: bool) -> Vec<PaRecord> {
    let mut r = rng(seed, game, BATTER, Purpose::BatterPerception, 2_000_000);
    let e = mu.model_error.unwrap_or(ModelError { spin_axis_deg: 0.0, spin_rate_rel: 0.0, efficiency: 0.0, speed: 0.0 });
    let mut mem = PitcherMemory::new(draw_offsets(&mu.arsenal, &e, &mut r));
    let mut out = Vec::new();
    let learn = |mem: &mut PitcherMemory, ty: PitchType, w: f64, r: &mut KeyedRng| {
        if !adapt {
            return;
        }
        let k = mu.arsenal.iter().position(|a| a.def.ty == ty).unwrap();
        let noise = CUE_NOISE.map(|s| s * r.truncated_normal(4.0));
        mem.learn(k, w, ETA, noise);
    };
    for t in 0..times {
        let sc = build_scouting_offsets(&mu.env, &mu.pitcher, &mu.arsenal, &mu.zone, &mem.offsets);
        let first_play = game * 100_000 + t * PA_PLAY_STRIDE;
        let mut count = Count::default();
        let mut pitches = Vec::new();
        let mut play = first_play;
        let result = loop {
            let bs: Vec<f64> = (0..mu.arsenal.len())
                .map(|k| if adapt { mem.break_sigma(k, mu.perception.break_sigma, ETA) } else { mu.perception.break_sigma })
                .collect();
            let prev = pitches.last().map(|p: &PitchRecord| p.read);
            let rec = simulate_pitch_ctx(mu, &sc, count, seed, play, &bs, prev);
            play += 1;
            // Learning uses the pitch type shown on the scoreboard after the pitch, a public
            // channel (agent-models §3.8 [C]), with the batter's own sighting noise.
            learn(&mut mem, rec.ty, 1.0, &mut r);
            let call = rec.call;
            pitches.push(rec);
            match call {
                Call::Ball => {
                    count.balls += 1;
                    if count.balls == 4 { break PaResult::Walk; }
                }
                Call::CalledStrike | Call::SwingingStrike | Call::FoulTip => {
                    count.strikes += 1;
                    if count.strikes == 3 { break PaResult::Strikeout; }
                }
                Call::Foul => if count.strikes < 2 { count.strikes += 1 },
                Call::HitByPitch => break PaResult::HitByPitch,
                Call::InPlay => break PaResult::InPlay,
            }
            if pitches.len() >= MAX_PA_PITCHES { break PaResult::Unresolved; }
        };
        let called_third = result == PaResult::Strikeout && pitches.last().map(|p| p.call == Call::CalledStrike).unwrap_or(false);
        out.push(PaRecord { pitches, result, called_third });
        // Watch the rest of the lineup: sample the pitcher's usage (even-count mix).
        for j in 0..on_deck_pitches {
            let mut rr = rng(seed, first_play + 500 + j, PITCHER, Purpose::PitchSelection, 0);
            let tot: f64 = mu.arsenal.iter().map(|a| a.usage[1]).sum();
            let mut u = rr.uniform() * tot;
            let mut ty = mu.arsenal[0].def.ty;
            for a in &mu.arsenal {
                if u < a.usage[1] { ty = a.def.ty; break; }
                u -= a.usage[1];
            }
            learn(&mut mem, ty, ON_DECK_WEIGHT, &mut r);
        }
    }
    out
}
