//! Scorer rulings checked against the text and examples of OBR Rule 9 (the comment in each
//! test names the clause). Each play is built by hand as a serialized play record, so no
//! number here comes from the scorer or the simulation.
//!
//! Review: docs/reviews/2026-11-14-gemini-scorer-review.md. Cases for its findings P1-1 to
//! P1-6 and P2-3 (the rest are in `scorer_cases.rs`, `accounting.rs` and `hand_scored.rs`).
use bb_schema::ev::{self, EventBody};
use bb_stats::scorer::{score_play, Outcome, PlayRuling};
use flatbuffers::{FlatBufferBuilder, UnionWIPOffset, WIPOffset};

const B: u16 = 10; // the batter
const R1: u16 = 11;
const R2: u16 = 12;
const R3: u16 = 13;

#[derive(Clone)]
enum Body {
    Out(u16, u8, ev::OutKind),
    Adv(u16, u8, u8, ev::AdvanceCause),
    Run(u16),
    Dead(ev::DeadBallReason),
    WalkOff,
}

#[derive(Clone)]
struct Play {
    result: ev::PaResult,
    /// Evidence is written (an A12 log). False: a log from before it existed.
    evidence: bool,
    /// Runners on at the pitch, batter at base 0.
    runners: Vec<(u16, u8)>,
    first_touch: Option<f64>,
    handler: Option<u16>,
    secured: Option<(u16, f64)>, // fielder, distance from home (m) straight up the middle
    margin: Option<f64>,
    eta: Option<f64>,
    /// (base, release time, attempted out, runner it was meant for)
    throws: Vec<(u8, f64, bool, u16)>,
    carries: Vec<(u8, u16)>,
    touches: Vec<(u16, u8, f64)>,
    body: Vec<Body>,
}

impl Play {
    /// A batted ball in play, batter alone on, nobody handled it yet.
    fn bip() -> Self {
        Play { result: ev::PaResult::Unknown, evidence: true, runners: vec![(B, 0)], first_touch: None, handler: None, secured: None, margin: None, eta: None, throws: vec![], carries: vec![], touches: vec![], body: vec![] }
    }
    fn on(mut self, runners: &[(u16, u8)]) -> Self {
        self.runners.extend_from_slice(runners);
        self
    }
    fn touch(mut self, t: f64) -> Self {
        self.first_touch = Some(t);
        self.touches.push((B, 1, t));
        self
    }
    fn handled(mut self, fielder: u16, at_m: f64) -> Self {
        self.handler = Some(fielder);
        self.secured = Some((fielder, at_m));
        self
    }
    fn margin(mut self, m: f64) -> Self {
        self.margin = Some(m);
        self
    }
    fn out(mut self, r: u16, base: u8, k: ev::OutKind) -> Self {
        self.body.push(Body::Out(r, base, k));
        self
    }
    fn adv(mut self, r: u16, from: u8, to: u8, c: ev::AdvanceCause) -> Self {
        self.body.push(Body::Adv(r, from, to, c));
        self
    }
    fn run(mut self, r: u16) -> Self {
        self.body.push(Body::Run(r));
        self
    }
    fn dead(mut self, d: ev::DeadBallReason) -> Self {
        self.body.push(Body::Dead(d));
        self
    }
    fn throw(mut self, base: u8, t: f64, attempted: bool, runner: u16) -> Self {
        self.throws.push((base, t, attempted, runner));
        self
    }
    fn bytes(&self) -> Vec<u8> {
        let mut f = FlatBufferBuilder::new();
        let mut bodies: Vec<(EventBody, WIPOffset<UnionWIPOffset>)> = Vec::new();
        if self.evidence {
            let runners: Vec<_> = self.runners.iter().map(|&(agent, base)| ev::RunnerOnBase::create(&mut f, &ev::RunnerOnBaseArgs { agent, base })).collect();
            let runners = f.create_vector(&runners);
            let throws: Vec<_> = self.throws.iter().map(|&(base, t, attempted_out, runner)| ev::ScorerThrow::create(&mut f, &ev::ScorerThrowArgs { base, t_release: t, attempted_out, runner, ..Default::default() })).collect();
            let throws = f.create_vector(&throws);
            let carries: Vec<_> = self.carries.iter().map(|&(base, runner)| ev::ScorerCarry::create(&mut f, &ev::ScorerCarryArgs { base, runner, ..Default::default() })).collect();
            let carries = f.create_vector(&carries);
            let touches: Vec<_> = self.touches.iter().map(|&(runner, base, t)| ev::ScorerTouch::create(&mut f, &ev::ScorerTouchArgs { runner, base, t })).collect();
            let touches = f.create_vector(&touches);
            let at = self.secured.map(|(_, d)| bb_schema::vec3d([0.0, d, 0.0]));
            let e = ev::ScorerEvidence::create(
                &mut f,
                &ev::ScorerEvidenceArgs {
                    batter: B,
                    runners: Some(runners),
                    has_first_touch: self.first_touch.is_some(),
                    first_touch_s: self.first_touch.unwrap_or_default(),
                    has_handling: self.handler.is_some(),
                    first_handler: self.handler.unwrap_or_default(),
                    has_secured: self.secured.is_some(),
                    reference_fielder: self.secured.map(|s| s.0).unwrap_or_default(),
                    secured_at: at.as_ref(),
                    has_margin: self.margin.is_some(),
                    clean_play_margin_s: self.margin.unwrap_or_default(),
                    has_batter_eta: self.eta.is_some(),
                    batter_eta_s: self.eta.unwrap_or_default(),
                    throws: Some(throws),
                    carries: Some(carries),
                    touches: Some(touches),
                    ..Default::default()
                },
            );
            bodies.push((EventBody::ScorerEvidence, e.as_union_value()));
        }
        let mut walk_off = false;
        for b in &self.body {
            match *b {
                Body::Out(runner, base, kind) => bodies.push((EventBody::RunnerOut, ev::RunnerOut::create(&mut f, &ev::RunnerOutArgs { runner, base, kind, ..Default::default() }).as_union_value())),
                Body::Adv(runner, from, to, cause) => bodies.push((EventBody::RunnerAdvanced, ev::RunnerAdvanced::create(&mut f, &ev::RunnerAdvancedArgs { runner, from_base: from, to_base: to, cause, ..Default::default() }).as_union_value())),
                Body::Run(runner) => bodies.push((EventBody::RunScored, ev::RunScored::create(&mut f, &ev::RunScoredArgs { runner, ..Default::default() }).as_union_value())),
                Body::Dead(reason) => bodies.push((EventBody::BallDead, ev::BallDead::create(&mut f, &ev::BallDeadArgs { reason }).as_union_value())),
                Body::WalkOff => walk_off = true,
            }
        }
        bodies.push((EventBody::PlateAppearanceEnded, ev::PlateAppearanceEnded::create(&mut f, &ev::PlateAppearanceEndedArgs { batter: B, result: self.result, ..Default::default() }).as_union_value()));
        if walk_off {
            bodies.push((EventBody::GameEnded, ev::GameEnded::create(&mut f, &ev::GameEndedArgs { reason: ev::GameEndReason::WalkOff, ..Default::default() }).as_union_value()));
        }
        let es: Vec<_> = bodies.into_iter().enumerate().map(|(i, (ty, b))| ev::Event::create(&mut f, &ev::EventArgs { seq: i as u32, body_type: ty, body: Some(b), ..Default::default() })).collect();
        let es = f.create_vector(&es);
        let p = ev::PlayRecord::create(&mut f, &ev::PlayRecordArgs { play_index: 1, events: Some(es) });
        f.finish(p, None);
        f.finished_data().to_vec()
    }
    fn judge(&self, outs_before: u32) -> PlayRuling {
        let bytes = self.bytes();
        score_play(flatbuffers::root::<ev::PlayRecord>(&bytes).unwrap(), outs_before, 0.15).unwrap()
    }
}

use ev::AdvanceCause as Cause;
use ev::OutKind as Kind;

// ---- P1-1: RBI and OBR 9.04 --------------------------------------------------------------

// 9.08(d): a sacrifice fly is scored when a runner scores after a caught fly with fewer than
// two out; 9.04(a)(1) credits the batter an RBI for it. A second out made on the bases (here
// the runner from first tagged out trying for second) does not take it away: 9.04(b)(1)
// withholds the RBI only for a force or reverse-force double play.
#[test]
fn sacrifice_fly_with_a_second_out_on_the_bases_keeps_the_rbi() {
    let p = Play::bip().on(&[(R1, 1), (R3, 3)]).handled(0x188, 75.0)
        .out(B, 0, Kind::FlyOut).run(R3).out(R1, 2, Kind::Tag);
    let r = p.judge(0);
    assert_eq!((r.outcome, r.sacrifice_fly, r.rbi), (Outcome::Out, true, 1));
}

// 9.04(a)(1): a run that scores on a safe hit is an RBI; 9.04(b)(1) makes no exception for
// runners put out on the bases afterwards.
#[test]
fn a_hit_that_scores_a_run_keeps_the_rbi_when_two_are_put_out_on_the_bases() {
    let p = Play::bip().on(&[(R2, 2), (R3, 3)]).handled(0x187, 70.0).touch(4.5).margin(-0.6)
        .run(R3).out(R2, 4, Kind::Tag).out(B, 2, Kind::Tag);
    let r = p.judge(0);
    assert_eq!((r.outcome, r.rbi), (Outcome::Hit(1), 1));
}

// 9.04(b)(1): no RBI when the batter grounds into a force double play.
#[test]
fn a_force_double_play_takes_the_rbi_away() {
    let p = Play::bip().on(&[(R1, 1), (R2, 2), (R3, 3)]).handled(0x186, 25.0)
        .run(R3).out(R2, 3, Kind::Force).out(B, 1, Kind::Force);
    // Bases loaded: R3 forced home too; the run scores before the two forces are made.
    assert_eq!(p.judge(0).rbi, 0);
}

// 9.04(b)(1): nor for a reverse-force double play: the first baseman steps on first (the batter
// is out), then tags R1, who was forced until the batter was put out.
#[test]
fn a_reverse_force_double_play_takes_the_rbi_away() {
    let p = Play::bip().on(&[(R1, 1), (R3, 3)]).handled(0x183, 27.0)
        .run(R3).out(B, 1, Kind::Force).out(R1, 2, Kind::Tag);
    assert_eq!(p.judge(0).rbi, 0);
}

// 9.04(b)(1) comment: a second out on a runner who was not forced (R2 caught off third with no
// one on first) is no force double play: the run that scored on the ground out is an RBI.
#[test]
fn a_ground_out_with_a_second_out_on_an_unforced_runner_keeps_the_rbi() {
    let p = Play::bip().on(&[(R2, 2), (R3, 3)]).handled(0x185, 25.0)
        .run(R3).out(B, 1, Kind::Force).out(R2, 3, Kind::Tag);
    assert_eq!(p.judge(0).rbi, 1);
}

// ---- P1-2: a throw out of play (OBR 5.06(b)(3)(D), 9.04(a), 9.08(d)) --------------------

// A runner who had touched home before the throw went out of play scored on the play; the
// engine writes his RunScored after BallDead, but the run is his and the batter's RBI. The
// runner from second, who scores only because bases were awarded, is not an RBI (9.04(a):
// "unaided by an error"; 9.12: the award follows a wild throw).
#[test]
fn runs_scored_before_a_throw_went_out_of_play_are_still_rbi() {
    let mut p = Play::bip().on(&[(R2, 2), (R3, 3)]).handled(0x186, 25.0).touch(4.4).margin(-0.5)
        .throw(1, 3.2, true, B)
        .dead(ev::DeadBallReason::OutOfPlay)
        .adv(R3, 3, 4, Cause::Award).run(R3).adv(R2, 2, 4, Cause::Award).run(R2);
    p.touches.push((R3, 4, 2.9)); // R3 crossed the plate before the throw
    let r = p.judge(0);
    assert_eq!(r.outcome, Outcome::Hit(1));
    assert_eq!(r.rbi, 1, "R3's run (touched home at 2.9 s) counts, R2's (an award) does not");
}

// 9.08(d): a sacrifice fly needs a run scored without the help of an error. A runner who
// scores only because a throw went out of play is no sacrifice fly and no RBI.
#[test]
fn a_run_given_by_an_errant_throw_is_not_a_sacrifice_fly() {
    let p = Play::bip().on(&[(R2, 2)]).handled(0x188, 75.0)
        .out(B, 0, Kind::FlyOut).throw(3, 6.0, true, R2).dead(ev::DeadBallReason::OutOfPlay)
        .adv(R2, 2, 4, Cause::Award).run(R2);
    let r = p.judge(0);
    assert_eq!((r.sacrifice_fly, r.rbi), (false, 0));
}

// 9.04(a)(1): a run that would have scored on the hit anyway is an RBI even if a wild throw
// also awarded it. Runner on third, single: third + one base = home.
#[test]
fn a_run_the_hit_would_have_scored_is_an_rbi_even_when_the_award_scores_it() {
    let p = Play::bip().on(&[(R3, 3)]).handled(0x187, 70.0).touch(4.5).margin(-0.6)
        .throw(2, 5.0, true, B).dead(ev::DeadBallReason::OutOfPlay)
        .adv(R3, 3, 4, Cause::Award).run(R3);
    let r = p.judge(0);
    assert_eq!((r.outcome, r.rbi), (Outcome::Hit(1), 1));
}

// ---- P1-3: a wild throw to first on a hit (9.05(a)(1), 9.12(a)(1)) --------------------

// 9.05(a)(1): a batter who reaches first on a fair ball he would have reached anyway is credited
// with a hit, even if the fielder's throw then goes wild. The reference throw to first would
// have been 0.35 s late: an infield hit; the wild throw is the fielder's error (not ruled yet)
// and gives the batter nothing in the hit's value.
#[test]
fn an_infield_hit_followed_by_a_wild_throw_is_still_a_hit() {
    let p = Play::bip().handled(0x186, 25.0).margin(-0.35)
        .throw(1, 3.1, true, B).dead(ev::DeadBallReason::OutOfPlay).adv(B, 0, 2, Cause::Award);
    assert_eq!(p.judge(0).outcome, Outcome::Hit(1), "the batter beat the ordinary throw");
    // The same, with his touch of first logged before the ball went dead.
    assert_eq!(p.clone().touch(4.2).judge(0).outcome, Outcome::Hit(1));
}

// 9.12(a)(1): had the ordinary throw retired him (it would have been 0.30 s early), the bases
// come from the thrower's error: he reaches on an error, no hit.
#[test]
fn a_wild_throw_that_beat_a_play_the_batter_would_have_lost_is_no_hit() {
    let p = Play::bip().handled(0x186, 25.0).margin(0.30)
        .throw(1, 3.1, true, B).dead(ev::DeadBallReason::OutOfPlay).adv(B, 0, 2, Cause::Award);
    assert_eq!(p.judge(0).outcome, Outcome::ReachedOnThrowAward);
}

// ---- P1-4: walk-off hits (9.06(f)) ----------------------------------------------------------

// The game ends when the winning run scores, before the batter has touched first, with none
// out. He is not charged an out: the safe hit that drove in the winning run is his, at least a
// single. The scorer reads where he was running (batter_eta).
#[test]
fn a_walk_off_single_before_the_batter_reaches_first_is_a_hit() {
    let mut p = Play::bip().on(&[(R3, 3)]).handled(0x187, 70.0).margin(-0.5).run(R3);
    p.body.push(Body::WalkOff);
    p.eta = Some(4.4);
    let r = p.judge(0);
    assert_eq!((r.outcome, r.rbi), (Outcome::Hit(1), 1));
}

// 9.06(f): a walk-off hit is worth only as many bases as the runner who scores the winning run
// advances, and only if the batter runs that far. R2 scores (two bases): a batter who reached
// third (he touched all three) is credited with a double.
#[test]
fn a_walk_off_hit_is_worth_the_winning_runners_bases() {
    let mut p = Play::bip().on(&[(R2, 2)]).handled(0x188, 85.0).margin(-1.0)
        .touch(4.0).run(R2);
    p.touches.push((B, 2, 5.4));
    p.touches.push((B, 3, 6.8));
    p.body.push(Body::WalkOff);
    assert_eq!(p.judge(1).outcome, Outcome::Hit(2));
    // And the same for the runner from third: one base, a single.
    let mut q = Play::bip().on(&[(R3, 3)]).handled(0x188, 85.0).margin(-1.0).touch(4.0).run(R3);
    q.touches.push((B, 2, 5.4));
    q.body.push(Body::WalkOff);
    assert_eq!(q.judge(1).outcome, Outcome::Hit(1));
}

// 9.06(g): a home run in the last half-inning counts in full (every runner and the batter).
#[test]
fn a_walk_off_home_run_counts_in_full() {
    let mut p = Play::bip().on(&[(R2, 2)]).run(R2).run(B);
    p.result = ev::PaResult::HomeRun;
    p.body.push(Body::WalkOff);
    let r = p.judge(0);
    assert_eq!((r.outcome, r.rbi), (Outcome::Hit(4), 2));
}

// ---- P1-5: putouts of preceding runners (9.05(b)(2)) ----------------------------------------

// 9.05(b)(2): no hit when a runner is called out for failing to touch the first base to which he
// is advancing, whoever handled the ball (here an outfielder).
#[test]
fn an_appeal_out_at_the_next_base_denies_the_hit() {
    let p = Play::bip().on(&[(R1, 1)]).handled(0x188, 70.0).touch(4.5).margin(-0.4)
        .out(R1, 2, Kind::Appeal);
    assert_eq!(p.judge(0).outcome, Outcome::FieldersChoice);
}

// 9.05(b)(3): an infielder's putout of a runner advancing one base is no hit (existing case,
// here with the runner's two-base try, which is: a hit) ----
#[test]
fn a_runner_thrown_out_going_for_two_bases_by_an_infielder_does_not_cost_the_hit() {
    let p = Play::bip().on(&[(R1, 1)]).handled(0x186, 25.0).touch(4.5).margin(-0.4)
        .out(R1, 3, Kind::Tag);
    assert_eq!(p.judge(0).outcome, Outcome::Hit(1));
}

// ---- P1-6: whom the defense played on ---------------------------------------------------------

// 9.05(a)(6) needs a preceding runner. A throw to second meant for the batter-runner himself,
// bases empty, is no fielder's choice.
#[test]
fn a_play_on_the_batter_runner_is_not_a_fielders_choice() {
    let p = Play::bip().handled(0x187, 60.0).touch(4.5).margin(0.20)
        .throw(2, 5.2, true, B);
    assert_eq!(p.judge(0).outcome, Outcome::Hit(1));
}

// The same for a carry at the batter-runner.
#[test]
fn a_carry_at_the_batter_runner_is_not_a_fielders_choice() {
    let mut p = Play::bip().handled(0x187, 60.0).touch(4.5).margin(0.20);
    p.carries.push((2, B));
    assert_eq!(p.judge(0).outcome, Outcome::Hit(1));
}

// A throw at a preceding runner only after the batter has touched first (he was safe on his own
// legs: no choice was made over him).
#[test]
fn a_throw_at_a_runner_after_the_batter_was_safe_at_first_is_not_a_choice() {
    let p = Play::bip().on(&[(R2, 2)]).handled(0x186, 25.0).touch(4.5).margin(0.30)
        .throw(3, 5.0, true, R2);
    assert_eq!(p.judge(0).outcome, Outcome::Hit(1));
}

// ... and one made while the batter was still running, which the ordinary throw to first would
// have beaten by more than 0.15 s, is the choice 9.05(a)(6) describes.
#[test]
fn a_throw_at_a_preceding_runner_over_a_batter_who_would_have_been_out_is_a_choice() {
    let p = Play::bip().on(&[(R2, 2)]).handled(0x186, 25.0).touch(4.5).margin(0.30)
        .throw(3, 3.0, true, R2);
    assert_eq!(p.judge(0).outcome, Outcome::FieldersChoice);
}

// 9.06(b): a throw at the batter-runner at second does not cap his triple.
#[test]
fn a_throw_at_the_batter_runner_does_not_cap_his_triple() {
    let mut p = Play::bip().handled(0x188, 95.0).touch(4.6).margin(-1.0)
        .throw(2, 5.3, true, B);
    p.touches.push((B, 2, 6.8));
    p.touches.push((B, 3, 8.4));
    assert_eq!(p.judge(0).outcome, Outcome::Hit(3));
}

// ---- P2-3 and 9.08(d): logs from before the evidence ----------------------------------------

// A fly out with a run in a log that has no evidence: whether it was a sacrifice fly cannot be
// told, so the scorer says so instead of charging an at-bat.
#[test]
fn an_old_log_fly_out_with_a_run_is_not_ruled_an_at_bat() {
    let mut p = Play::bip().on(&[(R3, 3)]).out(B, 0, Kind::FlyOut).run(R3);
    p.evidence = false;
    assert_eq!(p.judge(0).outcome, Outcome::Undetermined);
}

// Evidence written with the first four fields only (no runner list): `has_first_touch` reads as
// false, which must not read as "the batter never reached first".
#[test]
fn an_early_evidence_table_does_not_turn_hits_into_outs() {
    let mut p = Play::bip();
    p.runners.clear();
    let r = p.judge(0);
    assert_eq!(r.outcome, Outcome::Undetermined);
}

// ---- 9.08(d): who handled the ball ------------------------------------------------------------

// "handled by an outfielder or an infielder running in the outfield": an outfielder who takes a
// shallow fly in on the grass has a sacrifice fly; a second baseman who catches one in the
// infield does not.
#[test]
fn a_shallow_fly_caught_by_an_outfielder_is_a_sacrifice_fly_one_by_an_infielder_is_not() {
    let by_of = Play::bip().on(&[(R3, 3)]).handled(0x188, 30.0).out(B, 0, Kind::FlyOut).run(R3);
    assert!(by_of.judge(0).sacrifice_fly);
    let by_if = Play::bip().on(&[(R3, 3)]).handled(0x184, 30.0).out(B, 0, Kind::FlyOut).run(R3);
    assert!(!by_if.judge(0).sacrifice_fly);
    // An infielder who runs out and takes it in the outfield: sacrifice fly again.
    let running_out = Play::bip().on(&[(R3, 3)]).handled(0x184, 60.0).out(B, 0, Kind::FlyOut).run(R3);
    assert!(running_out.judge(0).sacrifice_fly);
}
