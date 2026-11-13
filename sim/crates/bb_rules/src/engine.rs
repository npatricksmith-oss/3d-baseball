//! The rules step function (rules-spec §4–§7).

use crate::state::*;

/// Base distance along a foul line, m [S] (90 ft).
const BASE_DIST: f64 = 27.432;
/// Baseball radius, m (physics-spec §2).
const BALL_R: f64 = 0.036888;

/// Signed distance from the nearer foul line, positive into fair territory.
fn line_dist(x: f64, y: f64) -> f64 {
    (y - x.abs()) / core::f64::consts::SQRT_2
}

/// A ball touching the ground is fair if its contact point (under the center, on flat
/// ground) is on fair ground; the foul lines are fair territory, so the boundary is the
/// lines' outer edge (2026 OBR "Fair Ball" clarification; rules-spec §5.2).
fn fair_ground(x: f64, y: f64) -> bool {
    line_dist(x, y) >= 0.0
}

/// A ball in the air is over fair territory if any part of it is (rules-spec §5.2).
fn fair_air(x: f64, y: f64) -> bool {
    line_dist(x, y) >= -BALL_R
}

/// Distance along the nearer foul line from home (for "beyond first or third base").
fn along_line(x: f64, y: f64) -> f64 {
    (y + x.abs()) / core::f64::consts::SQRT_2
}

fn fact_time(input: &Input) -> Option<f64> {
    match *input {
        Input::StartGame => None,
        Input::PitchDelivered { t }
        | Input::PitchCrossed { t, .. }
        | Input::SwingMeasured { t, .. }
        | Input::BatContact { t }
        | Input::BallTouched { t, .. }
        | Input::BallPassedBase { t, .. }
        | Input::BallSettled { t, .. }
        | Input::PlayTimeout { t }
        | Input::GloveSecured { t, .. }
        | Input::BaseTouched { t, .. }
        | Input::BaseLeft { t, .. }
        | Input::TagContact { t, .. }
        | Input::PlayEndCheck { t } => Some(t),
    }
}

/// Apply one input; returns the events it produced. Inputs that do not fit the current state
/// are ignored with a `RulesWarning` (the orchestrator sent something out of order).
pub fn step(st: &mut RulesState, input: &Input) -> Vec<Event> {
    let mut ev = Vec::new();
    if st.status == Status::Final {
        if !matches!(input, Input::PlayEndCheck { .. } | Input::PlayTimeout { .. }) {
            ev.push(Event::RulesWarning { code: "game_over" });
        }
        return ev;
    }
    // Facts within a live play must not go backward in time (rules-spec §1.2).
    if let (PlayState::Live(lp), Some(t)) = (&mut st.play, fact_time(input)) {
        if t + 1e-9 < lp.last_t {
            ev.push(Event::RulesWarning { code: "time_reversed" });
        }
        lp.last_t = lp.last_t.max(t);
    }
    match *input {
        Input::StartGame => {
            if st.status != Status::Pregame {
                ev.push(Event::RulesWarning { code: "already_started" });
                return ev;
            }
            st.status = Status::InProgress;
            ev.push(Event::GameStarted);
            start_half(st, &mut ev);
        }
        Input::PitchDelivered { .. } => {
            if st.play != PlayState::Dead || st.status != Status::InProgress {
                ev.push(Event::RulesWarning { code: "pitch_while_live" });
                return ev;
            }
            st.play = PlayState::Pitch(PitchState::default());
        }
        Input::PitchCrossed { in_zone, bounced, .. } => {
            if let PlayState::Pitch(p) = &mut st.play {
                p.in_zone = Some(in_zone);
                p.bounced = bounced;
            }
        }
        Input::SwingMeasured { swung, .. } => {
            let PlayState::Pitch(mut p) = st.play else { return ev };
            if p.contact {
                return ev;
            }
            p.swung = Some(swung);
            judge_pitch(st, p, &mut ev);
        }
        Input::BallTouched { what: Touch::Batter, .. } => {
            let PlayState::Pitch(mut p) = st.play else { return ev };
            if p.contact {
                return ev;
            }
            p.hit_body = true;
            if p.swung.is_some() {
                judge_pitch(st, p, &mut ev);
            } else {
                // Wait for the swing measurement: a swing beats an HBP award (§4.4).
                st.play = PlayState::Pitch(p);
            }
        }
        Input::BatContact { .. } => {
            let PlayState::Pitch(_) = st.play else { return ev };
            st.play = PlayState::Live(new_live_play(st));
        }
        Input::BallTouched { t, what } => live_touch(st, t, what, &mut ev),
        Input::BallPassedBase { fair, .. } => {
            if let PlayState::Live(lp) = &mut st.play {
                if lp.fair == FairFoul::Pending {
                    decide(st, fair, &mut ev);
                }
            }
        }
        Input::BallSettled { x, y, .. } => {
            if let PlayState::Live(lp) = &st.play {
                if lp.fair == FairFoul::Pending {
                    decide(st, fair_ground(x, y), &mut ev);
                }
            }
        }
        Input::GloveSecured { t, fielder, x, y } => glove(st, t, fielder, x, y, &mut ev),
        Input::BaseTouched { t, agent, base, holding_ball } => base_touched(st, t, agent, base, holding_ball, &mut ev),
        Input::BaseLeft { t, agent, base } => {
            if let PlayState::Live(lp) = &mut st.play {
                if let Some(r) = lp.runners.iter_mut().find(|r| r.runner == agent && !r.out) {
                    if r.on_base == Some(base) {
                        r.on_base = None;
                    }
                    if base == r.start_base && r.left_start_at.is_none() {
                        r.left_start_at = Some(t);
                    }
                }
            }
        }
        Input::TagContact { t, runner, holding_ball, .. } => {
            if !holding_ball {
                return ev;
            }
            let PlayState::Live(lp) = &st.play else { return ev };
            let Some(r) = lp.runners.iter().find(|r| r.runner == runner && !r.out) else { return ev };
            // A runner who owes a retouch is liable to the appeal even while on another base
            // (rules-spec §5.6).
            if lp.caught && r.retouch_required && !r.retouched {
                let b = r.start_base;
                record_out(st, t, runner, b, OutKind::Appeal, &mut ev);
                return ev;
            }
            if r.scored_at.is_some() || r.on_base.is_some() {
                return ev;
            }
            let base = r.at + 1;
            // A tag on a runner still forced to that base is a force out for run counting
            // (rules-spec §5.4, §6.4).
            let kind = if r.forced_to == Some(base) && r.at < base { OutKind::Force } else { OutKind::Tag };
            record_out(st, t, runner, base, kind, &mut ev);
        }
        Input::PlayEndCheck { .. } => end_play(st, &mut ev),
        Input::PlayTimeout { t } => play_timeout(st, t, &mut ev),
    }
    // A winning run that touched home while the ball could still be caught or go foul was
    // held back by `try_walk_off`; once the ball is down fair (or caught), judge it again.
    if matches!(input, Input::BallTouched { .. } | Input::BallPassedBase { .. } | Input::BallSettled { .. } | Input::GloveSecured { .. }) {
        try_walk_off(st, &mut ev);
    }
    ev
}

/// Judge a pitch that was not put in play (rules-spec §4.4).
fn judge_pitch(st: &mut RulesState, p: PitchState, ev: &mut Vec<Event>) {
    st.play = PlayState::Dead;
    let swung = p.swung == Some(true);
    let call = if p.hit_body {
        if swung {
            PitchCall::SwingingStrike
        } else if p.in_zone == Some(true) {
            PitchCall::CalledStrike
        } else {
            PitchCall::HitByPitch
        }
    } else if swung {
        PitchCall::SwingingStrike
    } else if p.in_zone == Some(true) && !p.bounced {
        PitchCall::CalledStrike
    } else {
        PitchCall::Ball
    };
    apply_call(st, call, ev);
}

fn start_half(st: &mut RulesState, ev: &mut Vec<Event>) {
    st.outs = 0;
    st.bases = [None; 3];
    st.runs_by_inning[st.half.batting()].push(0);
    if st.inning == RulesState::LIVENESS_INNING && st.half == Half::Top {
        ev.push(Event::RulesWarning { code: "liveness_inning_30" });
    }
    ev.push(Event::HalfInningStarted { inning: st.inning, half: st.half });
    start_pa(st, ev);
}

fn start_pa(st: &mut RulesState, ev: &mut Vec<Event>) {
    let side = st.half.batting();
    st.batter = st.lineup[side][st.next_up[side]];
    st.balls = 0;
    st.strikes = 0;
    ev.push(Event::PlateAppearanceStarted { batter: st.batter });
}

/// Complete a plate appearance; then the game ends (walk-off), the half ends, or the next
/// batter comes up — exactly one of these.
fn end_pa(st: &mut RulesState, result: PaResult, ev: &mut Vec<Event>) {
    ev.push(Event::PlateAppearanceEnded { batter: st.batter, result });
    let side = st.half.batting();
    st.next_up[side] = (st.next_up[side] + 1) % 9;
    st.balls = 0;
    st.strikes = 0;
    if st.walk_off {
        finish(st, true, ev);
    } else if st.outs >= 3 {
        end_half(st, ev);
    } else {
        start_pa(st, ev);
    }
}

fn end_half(st: &mut RulesState, ev: &mut Vec<Event>) {
    let side = st.half.batting();
    let runs = *st.runs_by_inning[side].last().unwrap_or(&0);
    let lob = st.bases.iter().filter(|b| b.is_some()).count() as u8;
    ev.push(Event::HalfInningEnded { runs, left_on_base: lob });
    st.bases = [None; 3];
    let [away, home] = st.score();
    let last = st.inning >= st.options.innings;
    let over = match st.half {
        Half::Top => last && home > away,
        Half::Bottom => last && home != away,
    };
    if over {
        finish(st, false, ev);
        return;
    }
    match st.half {
        Half::Top => st.half = Half::Bottom,
        Half::Bottom => {
            st.half = Half::Top;
            st.inning = st.inning.saturating_add(1);
        }
    }
    start_half(st, ev);
}

fn finish(st: &mut RulesState, walk_off: bool, ev: &mut Vec<Event>) {
    st.status = Status::Final;
    st.play = PlayState::Dead;
    let [away, home] = st.score();
    ev.push(Event::GameEnded { away, home, innings: st.inning, walk_off });
}

/// Count a run now. In the bottom of the last inning or later, the run that gives the home
/// team the lead sets `walk_off`; later runs on the play do not count (rules-spec §4.1),
/// except on a home run (handled by its caller).
fn score_run(st: &mut RulesState, runner: AgentId, ev: &mut Vec<Event>) {
    if st.walk_off {
        return;
    }
    let side = st.half.batting();
    let cell = st.runs_by_inning[side].last_mut().unwrap();
    *cell = cell.saturating_add(1);
    ev.push(Event::RunScored { runner });
    let [away, home] = st.score();
    if st.half == Half::Bottom && st.inning >= st.options.innings && home > away {
        st.walk_off = true;
    }
}

/// Pitch result and count transition (rules-spec §4.4).
fn apply_call(st: &mut RulesState, call: PitchCall, ev: &mut Vec<Event>) {
    match call {
        PitchCall::Ball => st.balls += 1,
        PitchCall::CalledStrike | PitchCall::SwingingStrike => st.strikes += 1,
        PitchCall::Foul => {
            if st.strikes < 2 {
                st.strikes += 1
            }
        }
        PitchCall::InPlay | PitchCall::HitByPitch => {}
    }
    ev.push(Event::PitchResult { call, balls: st.balls, strikes: st.strikes });
    if call == PitchCall::HitByPitch {
        award_first(st, AdvanceCause::HitByPitch, ev);
        end_pa(st, PaResult::HitByPitch, ev);
    } else if st.balls >= 4 {
        award_first(st, AdvanceCause::Walk, ev);
        end_pa(st, PaResult::Walk, ev);
    } else if st.strikes >= 3 {
        // v1: strike three is caught (uncaught third strikes need the catcher model).
        st.outs += 1;
        ev.push(Event::RunnerOut { runner: st.batter, base: 0, kind: OutKind::Strikeout });
        end_pa(st, PaResult::Strikeout, ev);
    }
}

/// Batter to first; runners advance one base only if forced (rules-spec §7.1). The award is
/// completed in full before any walk-off ends the game.
fn award_first(st: &mut RulesState, cause: AdvanceCause, ev: &mut Vec<Event>) {
    let forced1 = st.bases[0].is_some();
    let forced2 = forced1 && st.bases[1].is_some();
    let forced3 = forced2 && st.bases[2].is_some();
    if forced3 {
        let r = st.bases[2].take().unwrap();
        ev.push(Event::RunnerAdvanced { runner: r, from: 3, to: 4, cause });
        score_run(st, r, ev);
    }
    if forced2 {
        let r = st.bases[1].take().unwrap();
        ev.push(Event::RunnerAdvanced { runner: r, from: 2, to: 3, cause });
        st.bases[2] = Some(r);
    }
    if forced1 {
        let r = st.bases[0].take().unwrap();
        ev.push(Event::RunnerAdvanced { runner: r, from: 1, to: 2, cause });
        st.bases[1] = Some(r);
    }
    ev.push(Event::RunnerAdvanced { runner: st.batter, from: 0, to: 1, cause });
    st.bases[0] = Some(st.batter);
}

fn new_live_play(st: &RulesState) -> LivePlay {
    let mut runners = vec![RunnerState {
        runner: st.batter,
        start_base: 0,
        last_base: 0,
        at: 0,
        on_base: None,
        forced_to: Some(1),
        retouch_required: false,
        left_start_at: None,
        retouched: false,
        out: false,
        scored_at: None,
    }];
    let occ = st.bases.map(|b| b.is_some());
    for (i, r) in st.bases.iter().enumerate() {
        if let Some(r) = r {
            let base = (i + 1) as Base;
            // Forced iff every base behind it is occupied (rules-spec §5.4).
            let forced = occ[..i].iter().all(|&o| o);
            runners.push(RunnerState {
                runner: *r,
                start_base: base,
                last_base: base,
                at: base,
                on_base: Some(base),
                forced_to: forced.then_some(base + 1),
                retouch_required: false,
                left_start_at: None,
                retouched: false,
                out: false,
                scored_at: None,
            });
        }
    }
    LivePlay {
        fair: FairFoul::Pending,
        grounded: false,
        first_touch: None,
        caught: false,
        runners,
        outs: Vec::new(),
        runs_pending: Vec::new(),
        last_t: 0.0,
    }
}

/// Decide fair/foul for a pending batted ball (rules-spec §5.2).
fn decide(st: &mut RulesState, fair: bool, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    if lp.fair != FairFoul::Pending {
        return;
    }
    ev.push(Event::FairFoulDecided { fair });
    if fair {
        lp.fair = FairFoul::Fair;
        apply_call(st, PitchCall::InPlay, ev);
    } else {
        lp.fair = FairFoul::Foul;
        if lp.caught {
            // A caught foul fly: the batter is already out; the play continues for tag-ups.
            return;
        }
        // Uncaught foul: dead ball, runners return (rules-spec §7.4).
        st.play = PlayState::Dead;
        apply_call(st, PitchCall::Foul, ev);
    }
}

fn live_touch(st: &mut RulesState, _t: f64, what: Touch, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    match what {
        Touch::Ground { x, y } => {
            lp.grounded = true;
            if lp.fair == FairFoul::Pending && along_line(x, y) > BASE_DIST {
                decide(st, fair_ground(x, y), ev);
            }
        }
        Touch::Wall { x, y } => {
            lp.grounded = true;
            if lp.fair == FairFoul::Pending {
                decide(st, fair_air(x, y), ev);
            }
        }
        Touch::FoulPole => {
            // The pole is fair; only a ball that has not touched anything is a home run.
            if lp.grounded {
                if lp.fair == FairFoul::Pending {
                    decide(st, true, ev);
                }
            } else {
                lp.fair = FairFoul::Fair;
                home_run(st, ev);
            }
        }
        Touch::OutOfPlay { x, y, in_flight } => {
            let fair = match lp.fair {
                FairFoul::Fair => true,
                FairFoul::Foul => false,
                FairFoul::Pending => {
                    if in_flight { fair_air(x, y) } else { fair_ground(x, y) }
                }
            };
            if !fair {
                if lp.fair == FairFoul::Pending {
                    decide(st, false, ev);
                } else {
                    st.play = PlayState::Dead;
                }
            } else if in_flight && !lp.grounded {
                home_run(st, ev);
            } else {
                if lp.fair == FairFoul::Pending {
                    decide(st, true, ev);
                }
                ground_rule_double(st, ev);
            }
        }
        Touch::Batter => {}
    }
}

fn home_run(st: &mut RulesState, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = st.play.clone() else { return };
    ev.push(Event::FairFoulDecided { fair: true });
    ev.push(Event::PitchResult { call: PitchCall::InPlay, balls: st.balls, strikes: st.strikes });
    st.play = PlayState::Dead;
    st.bases = [None; 3];
    // Every runner not put out scores exactly once, including any who already touched home
    // (pending); then the batter. A home run is the exception to the walk-off "only the
    // needed runs" rule (rules-spec §4.1).
    let mut order: Vec<&RunnerState> = lp.runners.iter().filter(|r| !r.out).collect();
    order.sort_by_key(|r| (r.scored_at.is_none(), core::cmp::Reverse(r.start_base)));
    let side = st.half.batting();
    for r in order {
        ev.push(Event::RunnerAdvanced { runner: r.runner, from: r.start_base, to: 4, cause: AdvanceCause::Award });
        let cell = st.runs_by_inning[side].last_mut().unwrap();
        *cell = cell.saturating_add(1);
        ev.push(Event::RunScored { runner: r.runner });
    }
    let [away, home] = st.score();
    if st.half == Half::Bottom && st.inning >= st.options.innings && home > away {
        st.walk_off = true;
    }
    end_pa(st, PaResult::HomeRun, ev);
}

fn ground_rule_double(st: &mut RulesState, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = st.play.clone() else { return };
    st.play = PlayState::Dead;
    // Two bases from the time of the pitch (rules-spec §7.2).
    let mut order: Vec<&RunnerState> = lp.runners.iter().filter(|r| !r.out).collect();
    order.sort_by_key(|r| core::cmp::Reverse(r.start_base));
    st.bases = [None; 3];
    for r in order {
        let to = (r.start_base + 2).min(4);
        ev.push(Event::RunnerAdvanced { runner: r.runner, from: r.start_base, to, cause: AdvanceCause::Award });
        if to == 4 {
            score_run(st, r.runner, ev);
        } else {
            st.bases[(to - 1) as usize] = Some(r.runner);
        }
    }
    end_pa(st, PaResult::BattedBall, ev);
}

fn glove(st: &mut RulesState, t: f64, fielder: AgentId, x: f64, y: f64, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    if !lp.grounded && !lp.caught {
        // Catch: the batter is out whether the fly is fair or foul (rules-spec §5.3).
        lp.caught = true;
        lp.first_touch = Some(t);
        for r in lp.runners.iter_mut() {
            r.forced_to = None;
            // Runners who left their base before the first touch must retouch it.
            if r.start_base > 0 && r.on_base != Some(r.start_base) {
                r.retouch_required = true;
            }
        }
        ev.push(Event::Catch { fielder });
        if lp.fair == FairFoul::Pending {
            lp.fair = if fair_air(x, y) { FairFoul::Fair } else { FairFoul::Foul };
            ev.push(Event::FairFoulDecided { fair: lp.fair == FairFoul::Fair });
            ev.push(Event::PitchResult { call: PitchCall::InPlay, balls: st.balls, strikes: st.strikes });
        }
        let b = st.batter;
        record_out(st, t, b, 0, OutKind::FlyOut, ev);
    } else if lp.fair == FairFoul::Pending {
        // A fielder touching a pending ground ball decides fair/foul there.
        decide(st, fair_ground(x, y), ev);
    }
}

fn base_touched(st: &mut RulesState, t: f64, agent: AgentId, base: Base, holding_ball: bool, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    if lp.fair == FairFoul::Foul && !lp.caught {
        return;
    }
    if holding_ball {
        // Force out: the forced runner is not on the base and has not passed it, which
        // includes a runner who reached it and retreated (rules-spec §5.4; OBR 5.09(b)(6)).
        if lp.fair == FairFoul::Fair && !lp.caught {
            if let Some(r) = lp.runners.iter().find(|r| !r.out && r.forced_to == Some(base) && r.at < base && r.on_base != Some(base)) {
                let id = r.runner;
                record_out(st, t, id, base, OutKind::Force, ev);
                return;
            }
        }
        // Appeal on leaving early on a caught fly (rules-spec §5.6).
        if lp.caught {
            if let Some(r) = lp.runners.iter().find(|r| !r.out && r.retouch_required && !r.retouched && r.start_base == base) {
                let id = r.runner;
                record_out(st, t, id, base, OutKind::Appeal, ev);
            }
        }
        return;
    }
    let first_touch = lp.first_touch;
    let Some(r) = lp.runners.iter_mut().find(|r| r.runner == agent && !r.out) else { return };
    let retouch = base == r.start_base && r.retouch_required && !r.retouched && first_touch.map(|ft| t >= ft).unwrap_or(false);
    if r.scored_at.is_some() {
        if !retouch {
            // Passing bases on the way back to retouch: nothing changes until he gets there.
            return;
        }
        // He touched home before the catch and has come back to retouch (rules-spec §5.3):
        // the pending run is cancelled, and he must touch home again for it to count. The
        // bases between are not re-required in reverse order (missed-base appeals are
        // deferred, §5.6), so his next home touch is the next base in order.
        r.scored_at = None;
        r.last_base = 3;
        lp.runs_pending.retain(|(_, id)| *id != agent);
    }
    if retouch {
        r.retouched = true;
        r.at = base;
        r.on_base = Some(base);
        return;
    }
    if base == r.last_base + 1 {
        r.last_base = base;
        r.at = base;
        r.on_base = Some(base);
        if base == 4 {
            // A pending run: it counts at settlement or a time play, unless this runner is
            // retired on appeal first (rules-spec §5.6, §6.4).
            r.scored_at = Some(t);
            r.on_base = None;
            let id = r.runner;
            lp.runs_pending.push((t, id));
            try_walk_off(st, ev);
        }
    } else if base <= r.last_base {
        // Returning to an earlier base (or re-touching): the runner is now there.
        r.at = base;
        r.on_base = Some(base);
    } else {
        ev.push(Event::RulesWarning { code: "base_skipped" });
    }
}

/// End the game at the winning home touch when that run can no longer be cancelled
/// (rules-spec §4.1): nobody can still be forced out for a third out, the batter is not
/// retirable before first for a third out, and no retouch is owed.
fn try_walk_off(st: &mut RulesState, ev: &mut Vec<Event>) {
    if !(st.half == Half::Bottom && st.inning >= st.options.innings) {
        return;
    }
    let PlayState::Live(lp) = &st.play else { return };
    let [away, home] = st.score();
    let pending = lp.runs_pending.len() as u16;
    if home + pending <= away {
        return;
    }
    // Not while the batted ball is unresolved: a catch would oblige every runner who has
    // left to retouch (a home touch before it is no run yet, rules-spec §5.3), and a ball
    // that goes foul sends everyone back (runner-return review #1).
    if !lp.caught && !(lp.grounded && lp.fair == FairFoul::Fair) {
        return;
    }
    let outs_left = 3 - st.outs.min(3);
    let open_force = lp.runners.iter().any(|r| !r.out && r.scored_at.is_none() && r.forced_to.map(|b| r.at < b).unwrap_or(false));
    let owes_retouch = lp.runners.iter().any(|r| !r.out && r.retouch_required && !r.retouched);
    // With two or more outs to spare, no single play can make the third out before this run.
    let irrevocable = !owes_retouch && (outs_left >= 3 || !open_force);
    if !irrevocable {
        return;
    }
    let mut runs = lp.runs_pending.clone();
    runs.sort_by(|a, b| a.0.total_cmp(&b.0));
    let lp_runners = lp.runners.clone();
    for (_, r) in runs {
        let from = lp_runners.iter().find(|x| x.runner == r).map(|x| x.start_base).unwrap_or(0);
        ev.push(Event::RunnerAdvanced { runner: r, from, to: 4, cause: AdvanceCause::OnPlay });
        score_run(st, r, ev);
    }
    st.play = PlayState::Dead;
    end_pa(st, PaResult::BattedBall, ev);
}

fn record_out(st: &mut RulesState, t: f64, runner: AgentId, base: Base, kind: OutKind, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    let Some(idx) = lp.runners.iter().position(|r| r.runner == runner && !r.out) else { return };
    let start = lp.runners[idx].start_base;
    lp.runners[idx].out = true;
    lp.runners[idx].on_base = None;
    lp.runners[idx].scored_at = None;
    // An out runner's earlier home touch no longer counts.
    lp.runs_pending.retain(|(_, r)| *r != runner);
    lp.outs.push((t, runner, kind));
    ev.push(Event::RunnerOut { runner, base, kind });
    // Force removal: runners ahead of an out runner lose their force (rules-spec §5.4).
    for r in lp.runners.iter_mut() {
        if r.start_base > start {
            r.forced_to = None;
        }
    }
    st.outs += 1;
    if st.outs >= 3 {
        third_out(st, t, start, kind, ev);
    }
}

/// Third out: decide which runs count (rules-spec §6.4), end the play and the half.
fn third_out(st: &mut RulesState, t: f64, start: Base, kind: OutKind, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = st.play.clone() else { return };
    let batter_before_first = start == 0 && lp.runners.iter().find(|r| r.start_base == 0).map(|r| r.last_base == 0).unwrap_or(false);
    // No run scores on a third out that is a force, the batter-runner before first, or a
    // caught fly (the batter is retired before reaching first) (OBR 5.08(a)).
    let no_runs = kind == OutKind::Force || kind == OutKind::FlyOut || batter_before_first;
    st.play = PlayState::Dead;
    if !no_runs {
        // Time play: runs whose home touch was delivered before this out count, including
        // equal timestamps (input order is the tie-break, rules-spec §1.2).
        // A preceding runner retired on appeal for the third out takes the runs of the
        // runners following him with him (OBR 5.09(d)); runners ahead of him keep theirs.
        let following = |id: AgentId| kind == OutKind::Appeal && lp.runners.iter().any(|x| x.runner == id && x.start_base < start);
        let mut runs: Vec<(f64, AgentId)> = lp.runs_pending.iter().cloned().filter(|(ts, id)| *ts <= t && !following(*id)).collect();
        runs.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, r) in runs {
            score_run(st, r, ev);
        }
    }
    // Left on base: the runners actually standing on bases now.
    st.bases = [None; 3];
    for r in &lp.runners {
        if !r.out && r.scored_at.is_none() && (1..=3).contains(&r.at) && r.on_base == Some(r.at) {
            st.bases[(r.at - 1) as usize] = Some(r.runner);
        }
    }
    end_pa(st, PaResult::BattedBall, ev);
}

/// The play has settled (rules-spec §5.8 item 3): count runs, place runners, next batter.
fn end_play(st: &mut RulesState, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = st.play.clone() else { return };
    if lp.fair == FairFoul::Pending {
        ev.push(Event::RulesWarning { code: "settle_while_pending" });
        return;
    }
    if lp.runners.iter().any(|r| !r.out && r.scored_at.is_none() && r.on_base.is_none()) {
        // Someone is between bases: the play is not settled; the orchestrator keeps going.
        ev.push(Event::RulesWarning { code: "runner_between_bases" });
        return;
    }
    close_play(st, &lp, ev);
}

/// Place runners where they stand, count pending runs, and complete the PA.
fn close_play(st: &mut RulesState, lp: &LivePlay, ev: &mut Vec<Event>) {
    st.play = PlayState::Dead;
    let mut runs = lp.runs_pending.clone();
    runs.sort_by(|a, b| a.0.total_cmp(&b.0));
    st.bases = [None; 3];
    for r in &lp.runners {
        if !r.out && r.scored_at.is_none() && (1..=3).contains(&r.at) {
            let slot = &mut st.bases[(r.at - 1) as usize];
            if slot.is_some() {
                ev.push(Event::RulesWarning { code: "two_runners_on_base" });
            }
            *slot = Some(r.runner);
            if r.at != r.start_base {
                ev.push(Event::RunnerAdvanced { runner: r.runner, from: r.start_base, to: r.at, cause: AdvanceCause::OnPlay });
            }
        }
    }
    for (_, r) in runs {
        let from = lp.runners.iter().find(|x| x.runner == r).map(|x| x.start_base).unwrap_or(0);
        ev.push(Event::RunnerAdvanced { runner: r, from, to: 4, cause: AdvanceCause::OnPlay });
        score_run(st, r, ev);
    }
    end_pa(st, PaResult::BattedBall, ev);
}

/// Play timeout (rules-spec §5.8 item 4): a liveness failure in the orchestrator. Runners go
/// back to the last base touched; a batter-runner who never reached first is placed there
/// with forced runners advancing, and the whole thing is reported.
fn play_timeout(st: &mut RulesState, _t: f64, ev: &mut Vec<Event>) {
    let PlayState::Live(mut lp) = st.play.clone() else { return };
    ev.push(Event::RulesWarning { code: "play_timeout" });
    if lp.fair == FairFoul::Pending {
        lp.fair = FairFoul::Fair;
        ev.push(Event::FairFoulDecided { fair: true });
        ev.push(Event::PitchResult { call: PitchCall::InPlay, balls: st.balls, strikes: st.strikes });
    }
    // Resolve occupancy from the batter outward: anyone at base 0 goes to first, pushing
    // forced runners along.
    let mut order: Vec<usize> = (0..lp.runners.len()).collect();
    order.sort_by_key(|&i| lp.runners[i].at);
    let mut taken = [false; 4];
    for &i in order.iter().rev() {
        let r = &mut lp.runners[i];
        if r.out || r.scored_at.is_some() {
            continue;
        }
        let mut b = r.at.max(1);
        while b <= 3 && taken[b as usize] {
            b += 1;
        }
        if b > 3 {
            r.scored_at = Some(lp.last_t);
            lp.runs_pending.push((lp.last_t, r.runner));
        } else {
            taken[b as usize] = true;
            r.at = b;
        }
    }
    let lp2 = lp.clone();
    close_play(st, &lp2, ev);
}
