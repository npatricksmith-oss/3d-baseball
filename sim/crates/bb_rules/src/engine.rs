//! The rules step function (rules-spec §4–§7).

use crate::state::*;

/// Base distance along a foul line, m [S] (90 ft).
const BASE_DIST: f64 = 27.432;
/// Baseball radius, m (physics-spec §2); fair if any part of the ball is over fair territory.
const BALL_R: f64 = 0.036888;

/// Fair territory test at ground point (x, y): signed distance from the nearer foul line,
/// positive into fair territory, ≥ −R (rules-spec §5.2).
fn fair_at(x: f64, y: f64) -> bool {
    (y - x.abs()) / core::f64::consts::SQRT_2 >= -BALL_R
}

/// Distance along the nearer foul line from home (for "beyond first or third base").
fn along_line(x: f64, y: f64) -> f64 {
    (y + x.abs()) / core::f64::consts::SQRT_2
}

/// Apply one input; returns the events it produced. Inputs that do not fit the current state
/// are ignored with a `RulesWarning` (the orchestrator sent something out of order).
pub fn step(st: &mut RulesState, input: &Input) -> Vec<Event> {
    let mut ev = Vec::new();
    if st.status == Status::Final {
        if !matches!(input, Input::PlayEndCheck { .. }) {
            ev.push(Event::RulesWarning { code: "game_over" });
        }
        return ev;
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
            let PlayState::Pitch(mut p) = st.play.clone() else { return ev };
            if p.contact {
                return ev;
            }
            p.swung = Some(swung);
            st.play = PlayState::Dead;
            let call = if swung {
                PitchCall::SwingingStrike
            } else if p.in_zone == Some(true) && !p.bounced {
                PitchCall::CalledStrike
            } else {
                PitchCall::Ball
            };
            apply_call(st, call, &mut ev);
        }
        Input::BallTouched { what: Touch::Batter, .. } => {
            let PlayState::Pitch(p) = st.play.clone() else { return ev };
            if p.contact {
                return ev;
            }
            st.play = PlayState::Dead;
            // rules-spec §4.4 item 1.
            let call = if p.swung == Some(true) {
                PitchCall::SwingingStrike
            } else if p.in_zone == Some(true) {
                PitchCall::CalledStrike
            } else {
                PitchCall::HitByPitch
            };
            apply_call(st, call, &mut ev);
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
                    decide(st, fair_at(x, y), &mut ev);
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
            let Some(r) = lp.runners.iter().find(|r| r.runner == runner) else { return ev };
            if r.out || r.scored_at.is_some() || r.on_base.is_some() {
                return ev;
            }
            let base = r.last_base + 1;
            record_out(st, t, runner, base, OutKind::Tag, &mut ev);
        }
        Input::PlayEndCheck { .. } => end_play(st, &mut ev),
    }
    ev
}

fn start_half(st: &mut RulesState, ev: &mut Vec<Event>) {
    st.outs = 0;
    st.bases = [None; 3];
    st.runs_by_inning[st.half.batting()].push(0);
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

fn end_pa(st: &mut RulesState, result: PaResult, ev: &mut Vec<Event>) {
    ev.push(Event::PlateAppearanceEnded { batter: st.batter, result });
    let side = st.half.batting();
    st.next_up[side] = (st.next_up[side] + 1) % 9;
    st.balls = 0;
    st.strikes = 0;
    if st.status == Status::Final {
        return;
    }
    if st.outs >= 3 {
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
            st.inning += 1;
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

/// Count a run now (awards and settled plays). Ends the game on a walk-off.
fn score_run(st: &mut RulesState, runner: AgentId, ev: &mut Vec<Event>) -> bool {
    let side = st.half.batting();
    *st.runs_by_inning[side].last_mut().unwrap() += 1;
    ev.push(Event::RunScored { runner });
    let [away, home] = st.score();
    if st.half == Half::Bottom && st.inning >= st.options.innings && home > away {
        finish(st, true, ev);
        return true;
    }
    false
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
        if st.status != Status::Final {
            end_pa(st, PaResult::HitByPitch, ev);
        }
    } else if st.balls >= 4 {
        award_first(st, AdvanceCause::Walk, ev);
        if st.status != Status::Final {
            end_pa(st, PaResult::Walk, ev);
        }
    } else if st.strikes >= 3 {
        // v1: strike three is caught (uncaught third strikes need the catcher model, M4b).
        st.outs += 1;
        ev.push(Event::RunnerOut { runner: st.batter, base: 0, kind: OutKind::Strikeout });
        end_pa(st, PaResult::Strikeout, ev);
    }
}

/// Batter to first; runners advance one base only if forced (rules-spec §7.1).
fn award_first(st: &mut RulesState, cause: AdvanceCause, ev: &mut Vec<Event>) {
    let forced1 = st.bases[0].is_some();
    let forced2 = forced1 && st.bases[1].is_some();
    let forced3 = forced2 && st.bases[2].is_some();
    if forced3 {
        let r = st.bases[2].take().unwrap();
        ev.push(Event::RunnerAdvanced { runner: r, from: 3, to: 4, cause });
        if score_run(st, r, ev) {
            return;
        }
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
    LivePlay { fair: FairFoul::Pending, grounded: false, first_touch: None, caught: false, runners, outs: Vec::new(), runs_pending: Vec::new() }
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
                decide(st, fair_at(x, y), ev);
            }
        }
        Touch::Wall { x, y } => {
            lp.grounded = true;
            if lp.fair == FairFoul::Pending {
                decide(st, fair_at(x, y), ev);
            }
        }
        Touch::FoulPole => {
            lp.fair = FairFoul::Fair;
            home_run(st, ev);
        }
        Touch::OutOfPlay { x, y, in_flight } => {
            let fair = match lp.fair {
                FairFoul::Fair => true,
                FairFoul::Foul => false,
                FairFoul::Pending => fair_at(x, y),
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
    // Runners score in order from third; the batter last. A home run is the exception to
    // the walk-off "only the needed runs" rule: everyone scores (rules-spec §4.1).
    let mut order: Vec<&RunnerState> = lp.runners.iter().filter(|r| !r.out && r.scored_at.is_none()).collect();
    order.sort_by_key(|r| core::cmp::Reverse(r.start_base));
    let side = st.half.batting();
    for r in order {
        ev.push(Event::RunnerAdvanced { runner: r.runner, from: r.start_base, to: 4, cause: AdvanceCause::Award });
        *st.runs_by_inning[side].last_mut().unwrap() += 1;
        ev.push(Event::RunScored { runner: r.runner });
    }
    let [away, home] = st.score();
    end_pa(st, PaResult::HomeRun, ev);
    if st.status != Status::Final && st.half == Half::Bottom && st.inning >= st.options.innings && home > away {
        finish(st, true, ev);
    }
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
            if score_run(st, r.runner, ev) {
                return;
            }
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
            lp.fair = if fair_at(x, y) { FairFoul::Fair } else { FairFoul::Foul };
            ev.push(Event::FairFoulDecided { fair: lp.fair == FairFoul::Fair });
            ev.push(Event::PitchResult { call: PitchCall::InPlay, balls: st.balls, strikes: st.strikes });
        }
        let b = st.batter;
        record_out(st, t, b, 0, OutKind::FlyOut, ev);
    } else if lp.fair == FairFoul::Pending {
        // A fielder touching a pending ground ball decides fair/foul there.
        decide(st, fair_at(x, y), ev);
    }
}

fn base_touched(st: &mut RulesState, t: f64, agent: AgentId, base: Base, holding_ball: bool, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    if lp.fair == FairFoul::Foul && !lp.caught {
        return;
    }
    if holding_ball {
        // Force out: the forced runner has not reached the base (rules-spec §5.4).
        if lp.fair == FairFoul::Fair && !lp.caught {
            if let Some(r) = lp.runners.iter().find(|r| !r.out && r.forced_to == Some(base) && r.last_base < base) {
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
    if r.scored_at.is_some() {
        return;
    }
    if base == r.start_base && r.retouch_required && first_touch.map(|ft| t >= ft).unwrap_or(false) {
        r.retouched = true;
        r.last_base = base;
        r.on_base = Some(base);
        return;
    }
    if r.retouch_required && !r.retouched && base > r.start_base {
        // Advancing without a legal retouch: still liable to the appeal; track position.
        r.on_base = Some(base);
        r.last_base = base;
        return;
    }
    if base == r.last_base + 1 {
        r.last_base = base;
        r.on_base = Some(base);
        if base == 4 {
            r.scored_at = Some(t);
            r.on_base = None;
            let id = r.runner;
            lp.runs_pending.push((t, id));
        }
    } else if base <= r.last_base {
        r.on_base = Some(base);
    } else {
        ev.push(Event::RulesWarning { code: "base_skipped" });
    }
}

fn record_out(st: &mut RulesState, t: f64, runner: AgentId, base: Base, kind: OutKind, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    let Some(idx) = lp.runners.iter().position(|r| r.runner == runner && !r.out) else { return };
    let start = lp.runners[idx].start_base;
    lp.runners[idx].out = true;
    lp.runners[idx].on_base = None;
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
        third_out(st, t, runner, start, kind, ev);
    }
}

/// Third out: decide which runs count (rules-spec §6.4), end the play and the half.
fn third_out(st: &mut RulesState, t: f64, _runner: AgentId, start: Base, kind: OutKind, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = st.play.clone() else { return };
    let batter_before_first = start == 0 && lp.runners.iter().find(|r| r.start_base == 0).map(|r| r.last_base == 0).unwrap_or(false);
    let no_runs = kind == OutKind::Force || (batter_before_first && kind != OutKind::FlyOut);
    st.play = PlayState::Dead;
    if !no_runs {
        let mut runs: Vec<(f64, AgentId)> = lp.runs_pending.iter().cloned().filter(|(ts, _)| *ts < t).collect();
        runs.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, r) in runs {
            if score_run(st, r, ev) {
                return;
            }
        }
    }
    let result = if lp.caught { PaResult::BattedBall } else { PaResult::BattedBall };
    end_pa(st, result, ev);
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
    st.play = PlayState::Dead;
    let mut runs = lp.runs_pending.clone();
    runs.sort_by(|a, b| a.0.total_cmp(&b.0));
    st.bases = [None; 3];
    for r in &lp.runners {
        if !r.out && r.scored_at.is_none() && (1..=3).contains(&r.last_base) {
            let slot = &mut st.bases[(r.last_base - 1) as usize];
            if slot.is_some() {
                ev.push(Event::RulesWarning { code: "two_runners_on_base" });
            }
            *slot = Some(r.runner);
            if r.last_base != r.start_base {
                ev.push(Event::RunnerAdvanced { runner: r.runner, from: r.start_base, to: r.last_base, cause: AdvanceCause::OnPlay });
            }
        }
    }
    for (_, r) in runs {
        let from = lp.runners.iter().find(|x| x.runner == r).map(|x| x.start_base).unwrap_or(0);
        ev.push(Event::RunnerAdvanced { runner: r, from, to: 4, cause: AdvanceCause::OnPlay });
        if score_run(st, r, ev) {
            return;
        }
    }
    end_pa(st, PaResult::BattedBall, ev);
}
