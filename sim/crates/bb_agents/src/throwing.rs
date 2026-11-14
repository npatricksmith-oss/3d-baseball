//! A fielder's throw: ratings, set-up time and what hurrying costs (agent-models §5.5, §10).
//!
//! Everything here is what a fielder knows about his own body: how long he needs to get set
//! for a throw to a given target, and how much slower and wilder the ball leaves his hand if
//! he lets go sooner. Decision code may use all of it. The noise itself is drawn in
//! [`crate::motor::execute_throw`].

use bb_math::{Vec3, m};
use bb_observe::Hand;

use crate::attributes::{linear_from_grade, ratio_from_grade};

/// `ArmStrength` (agent-models §10): maximum throw speed at grade 50, m/s, and the
/// population σ [E].
pub const ARM_STRENGTH_50_MPS: f64 = 37.0;
pub const ARM_STRENGTH_SIGMA_MPS: f64 = 2.2;
/// `ArmAccuracy`: throw direction σ at grade 50, degrees, and its ratio per 10 grades [E].
pub const ARM_ACCURACY_50_DEG: f64 = 1.2;
pub const ARM_ACCURACY_RHO: f64 = 1.2;
/// `Transfer`: catch to release at grade 50, s, and its ratio per 10 grades [E].
pub const TRANSFER_50_S: f64 = 0.75;
pub const TRANSFER_RHO: f64 = 1.08;

/// Maximum throw speed at an `ArmStrength` grade, m/s.
pub fn arm_strength(grade: f64) -> f64 {
    linear_from_grade(grade, ARM_STRENGTH_50_MPS, ARM_STRENGTH_SIGMA_MPS)
}
/// Throw direction σ at an `ArmAccuracy` grade, rad.
pub fn arm_accuracy(grade: f64) -> f64 {
    ratio_from_grade(grade, ARM_ACCURACY_50_DEG, ARM_ACCURACY_RHO).to_radians()
}
/// How a `Transfer` grade scales a catch-to-release time (1 at grade 50; less is quicker).
pub fn transfer_scale(grade: f64) -> f64 {
    ratio_from_grade(grade, 1.0, TRANSFER_RHO)
}

/// The quickest a throw can be let go, as a fraction of the normal catch-to-release for that
/// throw [E]: no step, arm only.
pub const RUSH_MIN_FRAC: f64 = 0.6;
/// Direction σ grows by this much at full hurry (k_rush), and by this much more when he was
/// also off balance (k_off) [E].
pub const K_RUSH: f64 = 1.0;
pub const K_OFF: f64 = 1.0;
/// Fraction of his arm's speed lost at full hurry (no stride behind the ball), and lost in
/// addition when off balance [E].
pub const RUSH_SPEED_LOSS: f64 = 0.10;
pub const OFF_SPEED_LOSS: f64 = 0.08;
/// Turning the body to throw: time per 90° of turn beyond (or short of) the turn a throw
/// straight ahead needs, s [E].
pub const TURN_S: f64 = 0.12;
/// Footwork to get balanced before a throw, s [E]: after fielding the ball while moving away
/// from the target faster than [`AWAY_MPS`], and after a dive (on top of getting up).
pub const AWAY_FOOTWORK_S: f64 = 0.2;
pub const DIVE_FOOTWORK_S: f64 = 0.3;
pub const AWAY_MPS: f64 = 2.0;

/// Extra time (negative: time saved) to turn for a throw along `to_target`, having taken the
/// ball facing `facing` (both horizontal; lengths do not matter). A thrower is side-on at
/// release, glove shoulder to the target. Facing the ball he is square, so a target straight
/// ahead takes a quarter turn (the normal case, already in his transfer time); a target off
/// his glove shoulder needs none; one off his throwing shoulder needs a half turn.
pub fn turn_extra_s(facing: Vec3, to_target: Vec3, throws: Hand) -> f64 {
    let (fl, tl) = (m::hypot(facing.x, facing.y), m::hypot(to_target.x, to_target.y));
    if fl < 1e-9 || tl < 1e-9 {
        return 0.0;
    }
    // His left is Z × facing; the glove is on the side opposite the throwing hand.
    let side = if throws == Hand::Right { 1.0 } else { -1.0 };
    let (gx, gy) = (-facing.y / fl * side, facing.x / fl * side);
    let cos = ((gx * to_target.x + gy * to_target.y) / tl).clamp(-1.0, 1.0);
    let quarter = core::f64::consts::FRAC_PI_2;
    TURN_S * (m::acos(cos) - quarter) / quarter
}

/// Footwork owed before a balanced throw along `to_target` by a fielder who took the ball
/// moving at `catch_vel` (`dove`: he took it on the ground from a dive), s. A dive replaces
/// the moving-away term: he gets up from a standstill.
pub fn footwork_s(catch_vel: Vec3, to_target: Vec3, dove: bool) -> f64 {
    if dove {
        return DIVE_FOOTWORK_S;
    }
    let tl = m::hypot(to_target.x, to_target.y);
    if tl < 1e-9 {
        return 0.0;
    }
    let away = -(catch_vel.x * to_target.x + catch_vel.y * to_target.y) / tl;
    if away > AWAY_MPS { AWAY_FOOTWORK_S } else { 0.0 }
}

/// The set-up for one throw, in seconds after the fielder has the ball and is on his feet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SetUp {
    /// The soonest he can let it go.
    pub earliest_s: f64,
    /// When he is set: balanced, with his stride behind the throw.
    pub set_s: f64,
    /// He owes footwork to get balanced (moving away, or after a dive).
    pub off_balance: bool,
}

impl SetUp {
    /// `base_s`: his catch-to-release for this kind of throw; `turn_s` from
    /// [`turn_extra_s`]; `footwork_s` from [`footwork_s`].
    pub fn new(base_s: f64, turn_s: f64, footwork_s: f64) -> Self {
        let normal = (base_s + turn_s).max(0.0);
        Self { earliest_s: RUSH_MIN_FRAC * normal, set_s: normal + footwork_s, off_balance: footwork_s > 0.0 }
    }

    /// How hurried a release `elapsed_s` into the set-up is: 1 at the earliest, falling
    /// linearly to 0 once he is set.
    pub fn hurry(&self, elapsed_s: f64) -> f64 {
        let span = self.set_s - self.earliest_s;
        if span <= 0.0 { 0.0 } else { ((self.set_s - elapsed_s) / span).clamp(0.0, 1.0) }
    }

    /// Direction σ multiplier of a release at `hurry` (the `F_situation` of agent-models
    /// §1.3 for a throw).
    pub fn sigma_mult(&self, hurry: f64) -> f64 {
        1.0 + hurry * (K_RUSH + if self.off_balance { K_OFF } else { 0.0 })
    }

    /// Fraction of his arm's speed a release at `hurry` leaves with.
    pub fn speed_mult(&self, hurry: f64) -> f64 {
        1.0 - hurry * (RUSH_SPEED_LOSS + if self.off_balance { OFF_SPEED_LOSS } else { 0.0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    // agent-models §10: the grade table's printed values.
    #[test]
    fn ratings_match_the_grade_table() {
        assert!(close(arm_strength(50.0), 37.0) && close(arm_strength(80.0), 43.6) && close(arm_strength(20.0), 30.4));
        assert!((arm_accuracy(80.0).to_degrees() - 0.69).abs() < 0.005 && (arm_accuracy(20.0).to_degrees() - 2.07).abs() < 0.005);
        assert!((0.75 * transfer_scale(80.0) - 0.60).abs() < 0.005 && (0.75 * transfer_scale(20.0) - 0.94).abs() < 0.005);
    }

    // agent-models §5.5, the turn: facing the plate (−Y), a right-hander's glove shoulder
    // (his left) points to +X. A target there needs no turn (a quarter turn less than
    // straight ahead), one to −X a half turn (a quarter more).
    #[test]
    fn the_turn_favours_the_glove_side() {
        let face = Vec3::new(0.0, -1.0, 0.0);
        let (left, right, ahead, behind) = (Vec3::new(3.0, 0.0, 0.0), Vec3::new(-3.0, 0.0, 0.0), Vec3::new(0.0, -9.0, 0.0), Vec3::new(0.0, 2.0, 0.0));
        assert!(close(turn_extra_s(face, left, Hand::Right), -0.12));
        assert!(close(turn_extra_s(face, right, Hand::Right), 0.12));
        assert!(close(turn_extra_s(face, ahead, Hand::Right), 0.0));
        assert!(close(turn_extra_s(face, behind, Hand::Right), 0.0));
        // A left-hander is the mirror image.
        assert!(close(turn_extra_s(face, left, Hand::Left), 0.12));
        assert!(close(turn_extra_s(face, right, Hand::Left), -0.12));
        // Halfway between ahead and the glove side: 45° less turn, 0.06 s saved.
        assert!(close(turn_extra_s(face, Vec3::new(1.0, -1.0, 0.0), Hand::Right), -0.06));
    }

    // The first baseman's throw to second, the reason left-handers play first: fielding a
    // grounder at (19, 22) facing the plate, second base (0, 38.8) is off a right-hander's
    // throwing shoulder and off a left-hander's glove shoulder.
    #[test]
    fn a_left_handed_first_baseman_turns_less_to_throw_to_second() {
        let at = Vec3::new(19.0, 22.0, 0.0);
        let face = Vec3::new(-at.x, -at.y, 0.0);
        let to = Vec3::new(0.0 - at.x, 38.8 - at.y, 0.0);
        let (r, l) = (turn_extra_s(face, to, Hand::Right), turn_extra_s(face, to, Hand::Left));
        assert!(r > 0.09 && l < -0.09 && close(r, -l), "right {r:.3}, left {l:.3}");
    }

    #[test]
    fn footwork_is_owed_moving_away_or_after_a_dive() {
        let to = Vec3::new(10.0, 0.0, 0.0);
        assert_eq!(footwork_s(Vec3::new(-5.0, 0.0, 0.0), to, false), 0.2);
        assert_eq!(footwork_s(Vec3::new(5.0, 0.0, 0.0), to, false), 0.0);
        assert_eq!(footwork_s(Vec3::new(-1.5, 6.0, 0.0), to, false), 0.0); // across, not away
        assert_eq!(footwork_s(Vec3::ZERO, to, true), 0.3);
        // A dive while moving away: 0.3, not 0.5 (he gets up from a standstill).
        assert_eq!(footwork_s(Vec3::new(-5.0, 0.0, 0.0), to, true), 0.3);
    }

    // The worked example of agent-models §5.5: an infielder (0.65 s) moving away.
    #[test]
    fn hurry_runs_from_the_earliest_release_to_set() {
        let s = SetUp::new(0.65, 0.0, 0.2);
        assert!(close(s.earliest_s, 0.39) && close(s.set_s, 0.85) && s.off_balance);
        assert!(close(s.hurry(0.39), 1.0) && close(s.hurry(0.85), 0.0) && close(s.hurry(0.62), 0.5));
        assert!(close(s.hurry(0.1), 1.0) && close(s.hurry(2.0), 0.0));
        assert!(close(s.sigma_mult(1.0), 3.0) && close(s.speed_mult(1.0), 0.82));
        assert!(close(s.sigma_mult(0.5), 2.0) && close(s.speed_mult(0.5), 0.91));
        // On balance: only the rush terms.
        let b = SetUp::new(0.65, 0.0, 0.0);
        assert!(close(b.set_s, 0.65) && !b.off_balance);
        assert!(close(b.sigma_mult(1.0), 2.0) && close(b.speed_mult(1.0), 0.90));
        assert!(close(b.sigma_mult(0.0), 1.0) && close(b.speed_mult(0.0), 1.0));
    }
}
