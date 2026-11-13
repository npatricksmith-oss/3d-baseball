//! Batting cage (M2): one pitcher, one batter, one pitch at a time, no rules.
//!
//! In M2 the batter perceives the pitch perfectly: the aim point is the true ball position
//! at the template's contact depth. M3 replaces this with the hypothesis bank
//! (agent-models §3.3). Placement deviations can be injected directly for deterministic
//! sweeps, or drawn by `motor::execute_swing`.

use bb_agents::motor::{ReleaseNoise, SwingNoise, execute_release, execute_swing};
use bb_agents::pitcher::{PitchDef, PitchIntent, PitcherAttrs, plan_release};
use bb_agents::swing::{BatterAttrs, IntendedSwing, Placement, SwingTemplate, apply_placement, plan_swing};
use bb_math::{KeyedRng, Purpose, RngKey};
use bb_physics::flight::Crossing;
use bb_physics::pitch::launch;
use bb_physics::{BallState, FlightEnv};
use bb_world::pitch::{BatModel, PitchResolution, resolve_pitch};

pub struct Cage {
    pub env: FlightEnv,
    pub pitcher: PitcherAttrs,
    pub batter: BatterAttrs,
    pub template: SwingTemplate,
    pub bat: BatModel,
}

pub const PITCHER_AGENT: u16 = 0x0100;
pub const BATTER_AGENT: u16 = 0x0001;

impl Cage {
    pub fn average() -> Self {
        Self {
            env: FlightEnv::default_env(),
            pitcher: PitcherAttrs::average_rhp(),
            batter: BatterAttrs::average_rhb(),
            template: SwingTemplate::default(),
            bat: BatModel::default_34in(),
        }
    }

    /// Noise-free release for an intent.
    pub fn release(&self, intent: &PitchIntent) -> BallState {
        let def = PitchDef::template(intent.ty);
        let plan = plan_release(&self.env, &self.pitcher, &def, intent).expect("pitch target is reachable");
        launch(plan.r, &plan.shape, plan.yaw, plan.elev)
    }

    /// Release with execution noise from the keyed RNG.
    pub fn release_noisy(&self, intent: &PitchIntent, seed: u64, play: u32, command_scale: f64) -> BallState {
        let def = PitchDef::template(intent.ty);
        let plan = plan_release(&self.env, &self.pitcher, &def, intent).expect("pitch target is reachable");
        let noise = ReleaseNoise::new(self.pitcher.command, def.command_mult, command_scale);
        let mut rng = KeyedRng::new(RngKey { game_seed: seed, play_index: play, agent_id: PITCHER_AGENT, purpose: Purpose::PitchRelease, sub_index: 0 });
        execute_release(&plan, &noise, &mut rng).state
    }

    /// Perfect perception: where the ball truly is at the template's contact depth, and when.
    pub fn aim_perfect(&self, release: BallState) -> Option<(bb_math::Vec3, f64)> {
        let tr = bb_physics::flight::fly(&self.env, release, 0.0, bb_physics::flight::DT_PITCH, &[Crossing::YBelow(-1.0)], 3.0);
        // Contact depth depends on the plate x; iterate once from the judgment-plane x.
        let tj = tr.find(Crossing::YBelow(bb_physics::zone::Y_JUDGE))?;
        let x = tr.at(tj).0.x;
        let yc = self.template.contact_y(&self.batter, x);
        let tc = tr.find(Crossing::YBelow(yc))?;
        Some((tr.at(tc).0, tc))
    }

    /// Perfect-perception swing plan: aim at where the ball truly is at the contact depth.
    pub fn plan_perfect(&self, release: BallState, attack_angle: f64) -> Option<IntendedSwing> {
        let (aim, tc) = self.aim_perfect(release)?;
        Some(plan_swing(&self.batter, &self.template, &self.bat.profile, &self.bat.mass, self.bat.contact.ss_from_tip, aim, tc, attack_angle))
    }

    /// Resolve a pitch against a swing with an explicit placement deviation.
    pub fn swing_with(&self, release: BallState, placement: &Placement) -> PitchResolution {
        let plan = self.plan_perfect(release, self.template.attack_angle).expect("pitch reaches the batter");
        let swing = apply_placement(&plan, placement);
        resolve_pitch(&self.env, release, Some(&swing), &self.bat)
    }

    /// Resolve a pitch against a swing with execution noise.
    pub fn swing_noisy(&self, release: BallState, seed: u64, play: u32, k: f64) -> PitchResolution {
        let plan = self.plan_perfect(release, self.template.attack_angle).expect("pitch reaches the batter");
        let noise = SwingNoise::new(&self.batter, k);
        let mut rng = KeyedRng::new(RngKey { game_seed: seed, play_index: play, agent_id: BATTER_AGENT, purpose: Purpose::SwingExecution, sub_index: 0 });
        let ex = execute_swing(&plan, &noise, &mut rng);
        resolve_pitch(&self.env, release, Some(&ex.swing), &self.bat)
    }
}
