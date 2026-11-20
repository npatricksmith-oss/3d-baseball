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

/// Speed for accuracy on a set throw (agent-models §5.5): the lowest effort a fielder throws
/// at, as a fraction of his arm. Freeston, Adams and Rooney (2014; docs/calibration/
/// 2026-11-14-sources.md) measured elite young throwers at 80% and 100% of their maximum:
/// 26.4 against 33.0 m/s (0.80), so 0.80 is the measured range's floor.
pub const EFFORT_MIN: f64 = 0.80;
/// Their total target error at that effort against full: 46.9 against 61.1 cm.
pub const EFFORT_MIN_ERROR_RATIO: f64 = 46.9 / 61.1;

/// Direction σ multiplier of a throw at `effort` (fraction of his arm): 1 at full effort,
/// [`EFFORT_MIN_ERROR_RATIO`] at [`EFFORT_MIN`], linear between (the two measured points).
pub fn effort_sigma_mult(effort: f64) -> f64 {
    let e = effort.clamp(EFFORT_MIN, 1.0);
    1.0 - (1.0 - e) / (1.0 - EFFORT_MIN) * (1.0 - EFFORT_MIN_ERROR_RATIO)
}
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
/// A fielder's braking limit, m/s² (agent-models §5.3, `BRAKE`): the mean of measured maximal
/// decelerations of team-sport athletes, 4.2–5.0 m/s² (Harper, Cohen, Carling & Kiely 2020,
/// *Sports* 8(6), 76). The movement law and the plant before a throw share it.
pub const BRAKE_MPS2: f64 = 4.75;
/// The catch sets up the throw (agent-models §5.5, owner's Q32): the part of his catch-to-
/// release during which a fielder can already be braking (the ball going from glove to hand
/// while his feet stop), s [E, unsourced]. 2 m/s ÷ `BRAKE_MPS2`: a man under 2 m/s off the
/// line to his target owes nothing, as under the old flat rule.
pub const PLANT_OVERLAP_S: f64 = 2.0 / BRAKE_MPS2;
/// Footwork to get up and balanced after taking the ball on the ground from a dive, s [E].
pub const DIVE_FOOTWORK_S: f64 = 0.3;
/// Momentum toward the target (agent-models §5.5): the speed of a thrower's body toward his
/// target at release in a throw with a run-up (a crow hop), m/s: 3.0 m/s in elite team-handball
/// standing throws with run-up, against 1.5 m/s without (Wagner, Pfusterschmied, von Duvillard
/// & Müller 2011, *J Sports Sci Med* 10, 73, Table 1; read). `ArmStrength` (Statcast's top
/// throws, crow hops) is a throw at this speed; a crow hop from a stand reaches it inside his
/// `Transfer`.
pub const HOP_MPS: f64 = 3.0;
/// Ball speed gained per m/s of body speed toward the target beyond [`HOP_MPS`]: in the same
/// study the run-up's +1.5 m/s of body speed gave +1.6 m/s of ball (22.3 → 23.9 m/s; read).
/// Javelin release speed is likewise the run-up's speed plus the thrower's (Bartlett & Best
/// 1988, cited there).
pub const MOMENTUM_GAIN: f64 = 1.6 / 1.5;
/// The most body speed toward the target a thrower can carry into the block, m/s [E,
/// unsourced]: the gain is then at most 2.1 m/s (4.8 mph), inside the 6.1 mph median spread
/// between an outfielder's hardest throw and his top-10% mean (Savant 2025 arm-strength CSV;
/// docs/calibration/2026-11-19-throw-race.md §4).
pub const HOP_MAX_MPS: f64 = 5.0;

/// A4 decision parameters [E], agent-models §5.5–5.7. These configure beliefs and
/// option admission, never physical outcomes or execution noise.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChoiceParams {
    pub moving_mps: f64,
    pub on_bag_m: f64,
    pub carry_max_m: f64,
    pub tag_reach_m: f64,
    pub continuation_weight: f64,
}

impl Default for ChoiceParams {
    fn default() -> Self {
        Self { moving_mps: 0.5, on_bag_m: 0.05, carry_max_m: 8.0, tag_reach_m: 1.2, continuation_weight: 1.0 }
    }
}

impl ChoiceParams {
    /// Moving destination from a fielder's read of PUBLIC motion only. Arrival
    /// on a bag is not evidence of an advance toward the next one. The caller
    /// may supply noisy estimates; no intention or world object enters here.
    pub fn runner_target(self, path_m: f64, velocity_mps: f64, base_length_m: f64) -> Option<u8> {
        if velocity_mps.abs() < self.moving_mps || (path_m / base_length_m - (path_m / base_length_m).round()).abs() * base_length_m <= self.on_bag_m {
            return None;
        }
        let target = if velocity_mps > 0.0 { (path_m / base_length_m).floor() + 1.0 } else { (path_m / base_length_m).ceil() - 1.0 };
        (target >= 1.0 && target <= 4.0).then_some(target as u8)
    }
}

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

/// The plant owed before a set throw along `to_target` by a fielder who took the ball moving
/// at `catch_vel` (`dove`: he took it on the ground from a dive), s (agent-models §5.5, the
/// catch sets up the throw). Whatever of his motion is not toward the target (moving away, or
/// across the line) must be stopped under his braking limit before he can plant his back foot
/// and step to it; the part of that stop inside the glove-to-hand of his transfer
/// ([`PLANT_OVERLAP_S`]) costs nothing. A dive replaces it: he gets up from a standstill.
pub fn footwork_s(catch_vel: Vec3, to_target: Vec3, dove: bool) -> f64 {
    if dove {
        return DIVE_FOOTWORK_S;
    }
    (off_line_mps(catch_vel, to_target) / BRAKE_MPS2 - PLANT_OVERLAP_S).max(0.0)
}

/// His speed off the line to his target: the part of `vel` (horizontal) that is not toward
/// `to_target`, m/s (moving away counts whole; moving across counts its across part).
pub fn off_line_mps(vel: Vec3, to_target: Vec3) -> f64 {
    let tl = m::hypot(to_target.x, to_target.y);
    if tl < 1e-9 {
        return m::hypot(vel.x, vel.y);
    }
    let (ux, uy) = (to_target.x / tl, to_target.y / tl);
    let along = vel.x * ux + vel.y * uy;
    let across = vel.x * uy - vel.y * ux;
    m::hypot(along.min(0.0), across)
}

/// Ball speed a thrower adds by coming through the catch toward his target at `vel`, m/s
/// (agent-models §5.5): [`MOMENTUM_GAIN`] per m/s of his speed toward it beyond the crow hop's
/// [`HOP_MPS`], up to [`HOP_MAX_MPS`]. Nothing below that: a set throw from a slower approach
/// (or from a plant) has his crow hop and his `ArmStrength`. No accuracy cost (Wagner et al.
/// 2011: mean radial error 0.30 m with and without the run-up).
pub fn momentum_gain_mps(vel: Vec3, to_target: Vec3) -> f64 {
    let tl = m::hypot(to_target.x, to_target.y);
    if tl < 1e-9 {
        return 0.0;
    }
    let along = (vel.x * to_target.x + vel.y * to_target.y) / tl;
    MOMENTUM_GAIN * (along.min(HOP_MAX_MPS) - HOP_MPS).max(0.0)
}

/// The double-play pivot's catch-to-release at a `Transfer` grade of 50, s (agent-models
/// §5.5) [E]: the middle of the 0.7–0.9 s coaching norm (low confidence); scaled by the
/// `Transfer` grade it is 0.69 s at grade 70, near the one measured elite pivot (J.J. Hardy,
/// 0.67 s, FanGraphs 2014). No split by pivot man or feed type is known.
pub const PIVOT_S_50: f64 = 0.80;
/// An infielder's `Transfer` at grade 50, s (`bb_game::play::FielderAttrs::average`): the
/// pivot time scales with his own `Transfer` against this.
pub const INFIELD_TRANSFER_50_S: f64 = 0.65;
/// What the runner from first's slide costs the pivot man (A6, not built): time, s, and a
/// multiplier on his throw's direction σ. The game supplies zero slide clearance (TASKS.md, A6).
pub const PIVOT_SLIDE_DELAY_S: f64 = 0.0;
pub const PIVOT_SLIDE_SIGMA_MULT: f64 = 1.0;

/// A pivot man's own catch-to-release before turn and footwork, from his `Transfer` (s).
pub fn pivot_s(transfer_s: f64) -> f64 {
    PIVOT_S_50 * transfer_s / INFIELD_TRANSFER_50_S
}

/// The relay man's catch-to-release at a `Transfer` grade of 50, s (agent-models §5.5, the
/// relay man's exchange) [E]: an MLB.com analyst's expectation for a big-league shortstop's
/// relay throw home (0.75 s, "Rays' relay even more amazing under dissection"; snippet). Scaled
/// by the `Transfer` grade it is 0.64 s at grade 70, near the one measured elite relay exchange
/// (Tatis Jr., 0.62 s, MLB.com top defensive plays of 2021; snippet). It includes his turn to
/// his next target. No split by relay man or by the base he throws to is known.
pub const RELAY_S_50: f64 = 0.75;

/// A relay man's own catch-to-release, turn included, from his `Transfer` (s).
pub fn relay_s(transfer_s: f64) -> f64 {
    RELAY_S_50 * transfer_s / INFIELD_TRANSFER_50_S
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

    /// A double-play pivot (agent-models §5.5): the measured pivot times are already the quick
    /// release of a double-play race, so there is no earlier one. Earliest and set are both
    /// `base_s + turn + footwork` (and the slide's delay, nothing yet), at full arm.
    pub fn pivot(base_s: f64, turn_s: f64, footwork_s: f64) -> Self {
        let t = (base_s + turn_s).max(0.0) + footwork_s + PIVOT_SLIDE_DELAY_S;
        Self { earliest_s: t, set_s: t, off_balance: footwork_s > 0.0 }
    }

    /// A relay man's throw on (agent-models §5.5, the relay man's exchange): like the pivot,
    /// the measured relay exchanges are already the quick release of a relay race, so earliest
    /// and set are both `base_s + footwork` (the turn is inside `relay_s`), at full arm.
    pub fn relay(base_s: f64, footwork_s: f64) -> Self {
        let t = base_s.max(0.0) + footwork_s;
        Self { earliest_s: t, set_s: t, off_balance: footwork_s > 0.0 }
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

    /// agent-models §5.5: the pivot is 0.80 s at a grade-50 `Transfer` (0.65 s), and at grade
    /// 70 it comes within 0.02 s of the measured elite pivot, 0.67 s (0.80 × 1.08⁻² = 0.686).
    #[test]
    fn pivot_time_by_transfer_grade() {
        assert!((pivot_s(0.65) - 0.80).abs() < 1e-12);
        let g70 = pivot_s(0.65 * transfer_scale(70.0));
        assert!((g70 - 0.80 / (1.08f64 * 1.08)).abs() < 1e-9 && (g70 - 0.67).abs() < 0.02, "{g70}");
    }

    /// agent-models §5.5, the relay man's exchange: 0.75 s at grade 50 (the analyst's figure for
    /// a big-league shortstop), and at grade 70 within 0.03 s of the one measured elite relay
    /// exchange (Tatis Jr., 0.62 s); no early release.
    #[test]
    fn relay_time_by_transfer_grade() {
        // The value as set (the analyst's 0.75 s at grade 50).
        assert!((relay_s(0.65) - 0.75).abs() < 1e-12);
        // The sourced check: at grade 70 the grade rule lands within 0.03 s of the one measured
        // elite relay exchange (Tatis Jr., 0.62 s).
        let g70 = relay_s(0.65 * transfer_scale(70.0));
        assert!((g70 - 0.62).abs() < 0.03, "sourced check: grade 70 relay {g70:.3} s against the measured elite 0.62 s");
        let su = SetUp::relay(relay_s(0.65), 0.0);
        assert!(su.earliest_s == su.set_s && su.hurry(su.earliest_s) == 0.0 && su.speed_mult(0.0) == 1.0);
    }

    /// agent-models §5.5: a pivot has no earlier release than its set one, and a release then is
    /// at full arm and normal accuracy (the slide's seam adds nothing yet).
    #[test]
    fn a_pivot_is_released_set() {
        let su = SetUp::pivot(0.80, -0.06, 0.2);
        assert!((su.earliest_s - 0.94).abs() < 1e-12 && su.earliest_s == su.set_s);
        assert_eq!(su.hurry(su.earliest_s), 0.0);
        assert_eq!((su.speed_mult(0.0), su.sigma_mult(0.0)), (1.0, 1.0));
        assert_eq!((PIVOT_SLIDE_DELAY_S, PIVOT_SLIDE_SIGMA_MULT), (0.0, 1.0));
    }
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    // Freeston et al. 2014 (sources note): total target error 46.9 cm at 80% of maximum
    // speed against 61.1 cm at 100%. Halfway, the linear rule gives the mean of the two.
    #[test]
    fn effort_trades_speed_for_accuracy_as_measured() {
        assert!((effort_sigma_mult(1.0) - 1.0).abs() < 1e-12);
        assert!((effort_sigma_mult(0.80) - 46.9 / 61.1).abs() < 1e-12);
        assert!((effort_sigma_mult(0.90) - (46.9 + 61.1) / 2.0 / 61.1).abs() < 1e-12);
        assert!((effort_sigma_mult(0.5) - effort_sigma_mult(0.8)).abs() < 1e-12, "not below the measured range");
    }

    #[test]
    fn observed_runner_motion_admits_only_the_visible_leg() {
        // §5.6 independent geometry: 90-ft bases, third at 82.296 m. A man on
        // third offers no home play, even an arrival-speed read; leaving it does.
        let p = ChoiceParams::default();
        assert_eq!(p.runner_target(82.296, 0.0, 27.432), None);
        assert_eq!(p.runner_target(82.296, 8.0, 27.432), None);
        assert_eq!(p.runner_target(82.40, 8.0, 27.432), Some(4));
        assert_eq!(p.runner_target(70.0, -6.0, 27.432), Some(2));
        assert_eq!(p.runner_target(70.0, 6.0, 27.432), Some(3));
        assert_eq!(p.runner_target(70.0, 0.0, 27.432), None);
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

    // agent-models §5.5, the catch sets up the throw: the plant is the stop under the braking
    // limit less what fits inside the transfer. Independent figures: §5.3's stop from 7 m/s
    // (1.47 s at 4.75 m/s²), and the old rule's 2 m/s threshold, below which nothing is owed.
    #[test]
    fn the_plant_is_the_stop_under_the_braking_limit() {
        let to = Vec3::new(10.0, 0.0, 0.0);
        let stop_7 = footwork_s(Vec3::new(-7.0, 0.0, 0.0), to, false) + PLANT_OVERLAP_S;
        assert!((stop_7 - 1.47).abs() < 0.005, "{stop_7}");
        assert_eq!(footwork_s(Vec3::new(-2.0, 0.0, 0.0), to, false), 0.0);
        assert!(footwork_s(Vec3::new(-2.1, 0.0, 0.0), to, false) > 0.0);
        // Toward the target: nothing to stop, at any speed.
        assert_eq!(footwork_s(Vec3::new(7.0, 0.0, 0.0), to, false), 0.0);
        // Across the line counts as much as away; a diagonal counts its across part only.
        assert!(close(footwork_s(Vec3::new(0.0, 5.0, 0.0), to, false), footwork_s(Vec3::new(-5.0, 0.0, 0.0), to, false)));
        assert!(close(footwork_s(Vec3::new(4.0, 2.0, 0.0), to, false), 0.0));
        assert!(close(off_line_mps(Vec3::new(4.0, 3.0, 0.0), to), 3.0) && close(off_line_mps(Vec3::new(-4.0, 3.0, 0.0), to), 5.0));
        // A dive: his get-up from a standstill replaces it.
        assert_eq!(footwork_s(Vec3::ZERO, to, true), 0.3);
        assert_eq!(footwork_s(Vec3::new(-5.0, 0.0, 0.0), to, true), 0.3);
    }

    // Wagner et al. 2011 (Table 1): body speed toward the goal 1.5 → 3.0 m/s with the run-up gave
    // ball speed 22.3 → 23.9 m/s. A throw with the crow hop's 3.0 m/s is `ArmStrength` (gain 0);
    // 1.5 m/s more toward the target gives the study's 1.6 m/s; the gain stops at HOP_MAX_MPS.
    #[test]
    fn coming_through_the_catch_adds_the_measured_run_up_gain() {
        let to = Vec3::new(0.0, -60.0, 0.0);
        assert_eq!(momentum_gain_mps(Vec3::new(0.0, -3.0, 0.0), to), 0.0);
        assert_eq!(momentum_gain_mps(Vec3::ZERO, to), 0.0);
        assert_eq!(momentum_gain_mps(Vec3::new(0.0, 6.0, 0.0), to), 0.0);
        assert!((momentum_gain_mps(Vec3::new(0.0, -4.5, 0.0), to) - (23.9 - 22.3)).abs() < 1e-9);
        let cap = momentum_gain_mps(Vec3::new(0.0, -9.0, 0.0), to);
        assert!(close(cap, momentum_gain_mps(Vec3::new(0.0, -HOP_MAX_MPS, 0.0), to)) && cap * 2.2369 < 6.1, "{cap}");
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

    // agent-models §5.5, all three terms together: set = Transfer + turn + footwork, and he can
    // let go from RUSH_MIN_FRAC (0.6) of Transfer + turn (the footwork is not part of it).
    // Transfer 0.65 s:
    //   off his throwing shoulder (+0.12) and moving away at 5 m/s (plant 5/4.75 − 2/4.75 =
    //     0.6316): set = 0.65+0.12+0.6316 = 1.4016, earliest = 0.6·(0.65+0.12) = 0.462;
    //     at 0.6 s hurry = (1.4016−0.6)/(1.4016−0.462);
    //   off his glove shoulder (−0.12) after a dive (+0.3): set = 0.65−0.12+0.3 = 0.83,
    //     earliest = 0.6·0.53 = 0.318; half way (0.574 s) hurry 0.5: speed 1−0.5·0.18 = 0.91,
    //     direction σ ×(1+0.5·2) = 2.
    #[test]
    fn set_up_adds_transfer_turn_and_footwork() {
        let to = Vec3::new(10.0, 0.0, 0.0);
        let a = SetUp::new(0.65, 0.12, footwork_s(Vec3::new(-5.0, 0.0, 0.0), to, false));
        let plant = 3.0 / 4.75;
        assert!(close(a.set_s, 0.77 + plant) && close(a.earliest_s, 0.462) && a.off_balance);
        assert!(close(a.hurry(0.6), (0.17 + plant) / (0.308 + plant)));
        let b = SetUp::new(0.65, -0.12, footwork_s(Vec3::ZERO, to, true));
        assert!(close(b.set_s, 0.83) && close(b.earliest_s, 0.318) && b.off_balance);
        let mid = 0.5 * (0.318 + 0.83);
        assert!(close(b.hurry(mid), 0.5) && close(b.speed_mult(0.5), 0.91) && close(b.sigma_mult(0.5), 2.0));
        // A fielder taking a throw already facing his target needs 0.7 of the transfer and
        // no turn (§5.5): 0.7 · 0.65 = 0.455 s, earliest 0.273, on balance.
        let c = SetUp::new(0.7 * 0.65, 0.0, 0.0);
        assert!(close(c.set_s, 0.455) && close(c.earliest_s, 0.273) && !c.off_balance);
    }
}
