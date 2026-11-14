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
        | Input::BallReleasedFromGlove { t, .. }
        | Input::BaseTouched { t, .. }
        | Input::BaseLeft { t, .. }
        | Input::TagContact { t, .. }
        | Input::PlayEndCheck { t } => Some(t),
    }
}

/// Apply one input (a fact, command, or position sample); returns the events it produced.
/// Inputs that do not fit the current state are ignored with a `RulesWarning` (the
/// orchestrator sent something out of order).
pub fn step<'a>(st: &mut RulesState, fact: impl Into<Fact<'a>>) -> Vec<Event> {
    match fact.into() {
        Fact::Input(i) => step_input(st, i),
        Fact::Position(p) => step_position(st, p),
    }
}

/// Ground position of a base (1-3; 0 and 4 are home) in the sim frame.
fn base_xy(b: Base) -> (f64, f64) {
    let d = BASE_DIST / core::f64::consts::SQRT_2;
    match b {
        1 => (d, d),
        2 => (0.0, 2.0 * d),
        3 => (-d, d),
        _ => (0.0, 0.0),
    }
}

/// Progress along the base path, in bases (0 = home, 1 = first, ..., 4 = home again), of a
/// runner at ground point (x, y) whose last touched base is `at`: the nearest point on the
/// two base lines that meet at `at` (rules-spec §5.5).
fn path_progress(at: Base, x: f64, y: f64) -> f64 {
    let mut best = (f64::INFINITY, at as f64);
    for k in at.saturating_sub(1)..=at.min(3) {
        let (ax, ay) = base_xy(k);
        let (bx, by) = base_xy(k + 1);
        let (ux, uy) = (bx - ax, by - ay);
        // Not clamped below: a runner beyond a base on the far side from the next one (the
        // lead runner "beyond third base toward left field", OBR 5.09(b)(9) comment) is short
        // of it.
        let u = (((x - ax) * ux + (y - ay) * uy) / (ux * ux + uy * uy)).min(1.0);
        let (dx, dy) = (x - ax - u * ux, y - ay - u * uy);
        let d2 = dx * dx + dy * dy;
        if d2 < best.0 {
            best = (d2, k as f64 + u);
        }
    }
    best.1
}

/// A position sample (rules-spec §2 `RunnerPosition`): judges the overrun of first and
/// passing between bases (§5.5).
fn step_position(st: &mut RulesState, p: &RunnerPosition) -> Vec<Event> {
    let mut ev = Vec::new();
    if st.status == Status::Final {
        return ev;
    }
    let (attempt_m, margin) = (st.options.overrun_attempt_m, st.options.pass_margin_m / BASE_DIST);
    let PlayState::Live(lp) = &mut st.play else { return ev };
    if p.t + 1e-9 < lp.last_t {
        ev.push(Event::RulesWarning { code: "time_reversed" });
    }
    lp.last_t = lp.last_t.max(p.t);
    lp.sampled = true;
    let Some(i) = lp.runners.iter().position(|r| r.runner == p.runner && on_paths(r)) else { return ev };
    let r = &mut lp.runners[i];
    if r.start_base == 0 && r.at == 1 && r.last_base == 1 && r.on_base.is_none() && r.overrun != Overrun::Ended {
        if line_dist(p.x, p.y) > attempt_m {
            // Into fair territory toward second: an attempt; the protection is gone for good.
            r.overrun = Overrun::Ended;
        } else if r.overrun == Overrun::NotSeen && p.x > 0.0 && along_line(p.x, p.y) > BASE_DIST {
            r.overrun = Overrun::Protected;
        }
    }
    // A runner touching a base is at that base, wherever his center is within it; so is a
    // protected overrunning batter-runner, who is entitled to first (OBR 5.09(b)(4)).
    let s = match r.on_base {
        Some(b) => b as f64,
        None if r.overrun == Overrun::Protected => 1.0,
        None => path_progress(r.at, p.x, p.y),
    };
    r.sample = Some((p.t, s));
    // Compare with every runner whose position is known at this instant: standing on a base,
    // or sampled at the same time. Every pass is an out: one at a time in runner order,
    // re-reading the play after each, until none remain, the sampled runner is out, or the
    // inning ends (one retreat can leave several followers ahead of him).
    while let PlayState::Live(lp) = &st.play {
        if lp.runners[i].out {
            break;
        }
        let Some((fol, pre)) = pass_at_sample(lp, i, p.t, s, margin) else { break };
        let base = (lp.runners[fol].at + 1).min(4);
        passing_out(st, p.t, fol, pre, base, &mut ev);
    }
    ev
}

/// The first pass (passer, passed), in runner order, between runner `i`, sampled at `t` with
/// path progress `s`, and a runner whose position is known at `t` (rules-spec §5.5).
fn pass_at_sample(lp: &LivePlay, i: usize, t: f64, s: f64, margin: f64) -> Option<(usize, usize)> {
    for (j, o) in lp.runners.iter().enumerate() {
        if j == i || !on_paths(o) {
            continue;
        }
        let so = match (o.on_base, o.sample) {
            (Some(b), _) => b as f64,
            (None, Some((ts, so))) if (ts - t).abs() < 1e-9 => so,
            _ => continue,
        };
        let (fol, s_fol, pre, s_pre) = if lp.runners[i].start_base < o.start_base { (i, s, j, so) } else { (j, so, i, s) };
        if s_fol - s_pre > margin {
            return Some((fol, pre));
        }
    }
    None
}

/// A runner whose home touch is pending but who owes a retouch of his time-of-pitch base on a
/// caught fly (rules-spec §5.3): he is going back (or is liable to the appeal), so he is still
/// on the base paths, though his pending run stands until he retouches.
fn returning(r: &RunnerState) -> bool {
    r.scored_at.is_some() && r.retouch_required && !r.retouched
}

/// A runner who can pass or be passed (rules-spec §5.5): not out, and not home, unless he is
/// returning from a pending home touch.
fn on_paths(r: &RunnerState) -> bool {
    !r.out && (r.scored_at.is_none() || returning(r))
}

/// A pass counts once the batted ball is fair or caught; while it is in doubt the pass is held
/// (an uncaught foul ball is dead, and nothing the runners did meanwhile counts) (rules-spec
/// §5.5).
fn passing_judged(lp: &LivePlay) -> bool {
    lp.caught || lp.fair == FairFoul::Fair
}

/// Passing judged at a base touch (rules-spec §5.5): runner `i` is exactly on base `b`; every
/// other live runner is short of the base after the one he touched last and past the one
/// before it. Returns (passer, passed): the toucher if he passed a preceding runner, else the
/// first following runner (in runner order) he has fallen behind.
fn passed_on_touch(lp: &LivePlay, i: usize, b: Base) -> Option<(usize, usize)> {
    let si = lp.runners[i].start_base;
    let others = || lp.runners.iter().enumerate().filter(|&(j, o)| j != i && on_paths(o));
    // A preceding runner `o` is behind base `o.at + 1`, so behind `b` when b > o.at.
    if let Some((j, _)) = others().find(|(_, o)| si < o.start_base && b > o.at) {
        return Some((i, j));
    }
    // A following runner `o` is past base `o.at - 1 >= b`.
    others().find(|(_, o)| si > o.start_base && o.at > b).map(|(j, _)| (j, i))
}

/// Judge every pass that `agent`'s touch of `base` establishes (rules-spec §5.5): one at a
/// time in runner order, re-reading the play after each out, until none remain, the toucher
/// himself is out, or the inning ends. One retreat can leave several followers ahead of him.
fn judge_touch_passes(st: &mut RulesState, t: f64, agent: AgentId, base: Base, ev: &mut Vec<Event>) {
    loop {
        let PlayState::Live(lp) = &st.play else { return };
        let Some(i) = lp.runners.iter().position(|x| x.runner == agent && !x.out) else { return };
        let Some((fol, pre)) = passed_on_touch(lp, i, base) else { return };
        let hb = if fol == i { base } else { (lp.runners[fol].at + 1).min(4) };
        passing_out(st, t, fol, pre, hb, ev);
    }
}

/// Record a passing out: the following runner `fol` passed the preceding runner `pre`. While
/// the batted ball is in doubt the pass is held, with its time, and the passer leaves the play
/// at once; it is judged when the ball is ruled (`judge_pending_passes`).
fn passing_out(st: &mut RulesState, t: f64, fol: usize, pre: usize, base: Base, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    let (f, p) = (lp.runners[fol].runner, lp.runners[pre].runner);
    if !passing_judged(lp) {
        let r = &mut lp.runners[fol];
        r.out = true;
        r.on_base = None;
        lp.pending_passes.push(PendingPass { t, runner: f, passed: p, base });
        return;
    }
    ev.push(Event::RunnerPassed { runner: f, passed: p });
    record_out(st, t, f, base, OutKind::Tag, ev);
}

/// The batted ball is now fair or caught: every pass held while it was in doubt stands, each
/// recorded at its own time and in order, until none remain or the inning ends (rules-spec
/// §5.5, §6.4).
fn judge_pending_passes(st: &mut RulesState, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    let passes = core::mem::take(&mut lp.pending_passes);
    for pp in passes {
        if !matches!(st.play, PlayState::Live(_)) {
            return;
        }
        ev.push(Event::RunnerPassed { runner: pp.runner, passed: pp.passed });
        retire(st, pp.t, pp.runner, pp.base, OutKind::Tag, ev);
    }
}

fn step_input(st: &mut RulesState, input: &Input) -> Vec<Event> {
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
        Input::BallReleasedFromGlove { t, kind, infielder, .. } => {
            // Only throws matter so far: they fix what a throw out of play would award
            // (rules-spec §7.2). Drops and knocked balls have no producer yet.
            if let (ReleaseKind::Throw, PlayState::Live(lp)) = (kind, &mut st.play) {
                // Each runner's base at the release, as the rules hold it: one who owes a retouch
                // after a catch holds his own base until he has retouched it (rules-spec §5.3);
                // one who has legally touched home has scored; anyone else is at the base he
                // last touched (going back counts). (Gemini play-area review P1-3: the
                // high-water mark and early touches used to count.)
                let legal = |r: &RunnerState| if r.retouch_required && !r.retouched { r.start_base } else if r.scored_at.is_some() { 4 } else { r.at };
                let advanced = lp.runners.iter().filter(|r| !r.out).all(|r| legal(r) > r.start_base);
                let first_play = lp.throws == 0 && lp.outs.is_empty();
                let from_pitch = infielder && first_play && !advanced;
                // From the time of the pitch a runner counts from his base then, but a run
                // already legally scored stands.
                let from = lp.runners.iter().map(|r| (r.runner, if from_pitch && legal(r) < 4 { r.start_base } else { legal(r) })).collect();
                lp.throws = lp.throws.saturating_add(1);
                lp.throw = Some(ThrowAward { t, from_pitch, from });
            }
        }
        Input::BaseTouched { t, agent, base, holding_ball } => base_touched(st, t, agent, base, holding_ball, &mut ev),
        Input::BaseLeft { t, agent, base } => {
            if let PlayState::Live(lp) = &mut st.play {
                let sampled = lp.sampled;
                if let Some(r) = lp.runners.iter_mut().find(|r| r.runner == agent && !r.out) {
                    if r.on_base == Some(base) {
                        r.on_base = None;
                    }
                    // The batter-runner leaving first after his first touch of it: with samples in
                    // the play he is protected from now until a sample shows an attempt toward
                    // second, so a tag before his next sample is judged like one after it
                    // (rules-spec §5.5; OBR 5.09(b)(4) exception).
                    if sampled && base == 1 && r.start_base == 0 && r.at == 1 && r.last_base == 1 && r.overrun == Overrun::NotSeen {
                        r.overrun = Overrun::Protected;
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
            if r.scored_at.is_some() {
                return ev;
            }
            if let Some(b) = r.on_base {
                // A runner still forced onward has lost his right to the base he stands on,
                // alone on it or not: a tag retires him by force (rules-spec §5.4, §5.5; OBR
                // 5.09(b)(6)).
                if r.forced_to == Some(b + 1) {
                    record_out(st, t, runner, b + 1, OutKind::Force, &mut ev);
                    return ev;
                }
                // Two runners touching one base (rules-spec §5.5; OBR 5.06(a)(2), 5.06(b)(2)):
                // the preceding runner is entitled to it unless the following runner is forced
                // there (and then the preceding one was retired by force just above); the other
                // one is out when tagged.
                let Some(o) = lp.runners.iter().find(|o| o.runner != runner && !o.out && o.scored_at.is_none() && o.on_base == Some(b)) else { return ev };
                let (pre, fol) = if r.start_base > o.start_base { (r, o) } else { (o, r) };
                let entitled = if fol.forced_to == Some(b) { fol.runner } else { pre.runner };
                if entitled != runner {
                    record_out(st, t, runner, b, OutKind::Tag, &mut ev);
                }
                return ev;
            }
            if r.overrun == Overrun::Protected {
                // Overran first and is coming straight back (OBR 5.09(b)(4) exception).
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
    end_pa_lob(st, result, None, ev);
}

/// `end_pa`, with the left-on-base count of a third out made during a live play (counted from
/// runner identities, since two runners can share a base: rules-spec §6.4). `None` counts the
/// occupied bases.
fn end_pa_lob(st: &mut RulesState, result: PaResult, lob: Option<u8>, ev: &mut Vec<Event>) {
    ev.push(Event::PlateAppearanceEnded { batter: st.batter, result });
    let side = st.half.batting();
    st.next_up[side] = (st.next_up[side] + 1) % 9;
    st.balls = 0;
    st.strikes = 0;
    if st.walk_off {
        finish(st, true, ev);
    } else if st.outs >= 3 {
        end_half(st, lob, ev);
    } else {
        start_pa(st, ev);
    }
}

fn end_half(st: &mut RulesState, lob: Option<u8>, ev: &mut Vec<Event>) {
    let side = st.half.batting();
    let runs = *st.runs_by_inning[side].last().unwrap_or(&0);
    let lob = lob.unwrap_or_else(|| st.bases.iter().filter(|b| b.is_some()).count() as u8);
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
        overrun: Overrun::NotSeen,
        sample: None,
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
                overrun: Overrun::NotSeen,
                sample: None,
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
        pending_passes: Vec::new(),
        sampled: false,
        throws: 0,
        throw: None,
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
        judge_pending_passes(st, ev);
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
        Touch::OutOfPlay { .. } if lp.throw.is_some() => throw_out_of_play(st, ev),
        Touch::OutOfPlay { x, y, in_flight } => {
            let fair = match lp.fair {
                FairFoul::Fair => true,
                FairFoul::Foul => false,
                FairFoul::Pending => {
                    if in_flight { fair_air(x, y) } else { fair_ground(x, y) }
                }
            };
            if !fair {
                ev.push(Event::BallDead { reason: DeadBall::OutOfPlay });
                if lp.fair == FairFoul::Pending {
                    decide(st, false, ev);
                } else if lp.caught {
                    // A caught foul fly that then left the field without a throw: the batter is
                    // out and runners stay where they are.
                    let lp = lp.clone();
                    close_play(st, &lp, ev);
                } else {
                    st.play = PlayState::Dead;
                }
            } else if in_flight && !lp.grounded && (lp.fair == FairFoul::Pending || fair_air(x, y)) {
                home_run(st, ev);
            } else {
                // Includes a fly already ruled fair that a fielder deflected out over foul
                // ground: two bases (rules-spec §7.2; OBR 5.05(a)(8)).
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
    let PlayState::Live(_) = st.play else { return };
    ev.push(Event::FairFoulDecided { fair: true });
    ev.push(Event::PitchResult { call: PitchCall::InPlay, balls: st.balls, strikes: st.strikes });
    ev.push(Event::BallDead { reason: DeadBall::HomeRun });
    // A pass made before the ball left the park stands (rules-spec §5.5); passing on the trot
    // itself is deferred.
    judge_pending_passes(st, ev);
    let PlayState::Live(lp) = st.play.clone() else { return };
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
    ev.push(Event::BallDead { reason: DeadBall::GroundRuleDouble });
    // Two bases from the time of the pitch (rules-spec §7.2).
    let mut order: Vec<&RunnerState> = lp.runners.iter().filter(|r| !r.out).collect();
    order.sort_by_key(|r| core::cmp::Reverse(r.start_base));
    st.bases = [None; 3];
    for r in order {
        // A walk-off: the game ended with the winning run; nobody behind him is awarded
        // anything more (rules-spec §4.1; the batter's credit is the scorer's, OBR 9.06(f)).
        if st.walk_off {
            break;
        }
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

/// A thrown ball left the playing field (rules-spec §7.2): two bases from the time of the pitch
/// or of the throw, as fixed at the release; lead runner first, and a following runner never
/// reaches the base awarded to the runner ahead of him unless that runner scores.
fn throw_out_of_play(st: &mut RulesState, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = st.play.clone() else { return };
    let Some(award) = lp.throw.clone() else { return };
    ev.push(Event::BallDead { reason: DeadBall::OutOfPlay });
    if lp.fair == FairFoul::Pending {
        // Not reachable from the sim (a throw follows a fielder securing the ball, which rules
        // on the ball); a harness that sends one gets a fair ball.
        ev.push(Event::RulesWarning { code: "throw_while_pending" });
        ev.push(Event::FairFoulDecided { fair: true });
        apply_call(st, PitchCall::InPlay, ev);
    }
    st.play = PlayState::Dead;
    let mut order: Vec<(&RunnerState, Base)> = lp
        .runners
        .iter()
        .filter(|r| !r.out)
        .map(|r| (r, award.from.iter().find(|(id, _)| *id == r.runner).map(|&(_, b)| b).unwrap_or(r.at)))
        .collect();
    // Lead runner first (ties: the runner who started ahead).
    order.sort_by_key(|&(r, b)| (core::cmp::Reverse(b), core::cmp::Reverse(r.start_base)));
    st.bases = [None; 3];
    let mut ahead: Option<Base> = None;
    for (r, from) in order {
        // A walk-off ends the game at the winning run (rules-spec §4.1), as above.
        if st.walk_off {
            break;
        }
        let mut to = (from + 2).min(4);
        if let Some(a) = ahead
            && a < 4
        {
            to = to.min(a - 1).max(from);
        }
        ahead = Some(to);
        if to != r.start_base {
            // A runner who had legally scored before the throw scored on the play.
            let cause = if from == 4 { AdvanceCause::OnPlay } else { AdvanceCause::Award };
            ev.push(Event::RunnerAdvanced { runner: r.runner, from: r.start_base, to, cause });
        }
        if to == 4 {
            score_run(st, r.runner, ev);
        } else if to >= 1 {
            st.bases[(to - 1) as usize] = Some(r.runner);
        }
    }
    end_pa(st, PaResult::BattedBall, ev);
}

fn glove(st: &mut RulesState, t: f64, fielder: AgentId, x: f64, y: f64, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    // Secured: the ball is no longer a thrown ball (rules-spec §7.2).
    lp.throw = None;
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
        // Passes made while the fly was in the air came first (rules-spec §5.5).
        judge_pending_passes(st, ev);
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
            // On his way back to retouch (rules-spec §5.3) he is at this base now, and can pass
            // or be passed (§5.5); his pending run stands until he retouches.
            if returning(r) && base < 4 {
                r.at = base;
                r.on_base = Some(base);
                judge_touch_passes(st, t, agent, base, ev);
            }
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
        // A runner going back can leave following runners ahead of him (OBR 5.09(b)(9)
        // comment: passing by the lead runner's actions).
        judge_touch_passes(st, t, agent, base, ev);
        return;
    }
    if base > r.last_base + 1 {
        ev.push(Event::RulesWarning { code: "base_skipped" });
        return;
    }
    let i = lp.runners.iter().position(|x| x.runner == agent && !x.out).unwrap();
    if let Some((fol, pre)) = passed_on_touch(lp, i, base)
        && fol == i
    {
        // Reaching a base a preceding runner has not reached means passing him on the way: out
        // before the touch counts.
        passing_out(st, t, i, pre, base, ev);
        return;
    }
    // His touch counts. A preceding runner falling back behind following ones retires each of
    // them (OBR 5.09(b)(9) comment), judged after his touch so he stands on the base he
    // reached if one of those outs ends the inning.
    let r = &mut lp.runners[i];
    if r.start_base == 0 && base == 1 && r.last_base >= 1 {
        // The batter-runner back on first: any overrun protection is over.
        r.overrun = Overrun::Ended;
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
            // Nobody can be ahead of a runner at home.
            try_walk_off(st, ev);
            return;
        }
    } else {
        // Returning to an earlier base (or re-touching): the runner is now there.
        r.at = base;
        r.on_base = Some(base);
    }
    judge_touch_passes(st, t, agent, base, ev);
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
    let PlayState::Live(lp) = &st.play else { return };
    if lp.runners.iter().any(|r| r.runner == runner && !r.out) {
        retire(st, t, runner, base, kind, ev);
    }
}

/// Put `runner` out at time `t`. Unlike `record_out` it also takes a runner already off the
/// play awaiting a held pass (`judge_pending_passes`).
fn retire(st: &mut RulesState, t: f64, runner: AgentId, base: Base, kind: OutKind, ev: &mut Vec<Event>) {
    let PlayState::Live(lp) = &mut st.play else { return };
    let Some(idx) = lp.runners.iter().position(|r| r.runner == runner) else { return };
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
    // Left on base: the runners actually standing on bases now, counted by identity (two runners
    // sharing a base are both left on it, OBR 5.06(a)(2)); the one-slot-per-base `bases` keeps
    // the later one for the record.
    st.bases = [None; 3];
    let mut lob = 0u8;
    for r in &lp.runners {
        // An overrunning batter-runner still protected is entitled to first (§5.5).
        if !r.out && r.scored_at.is_none() && (1..=3).contains(&r.at) && (r.on_base == Some(r.at) || r.overrun == Overrun::Protected) {
            st.bases[(r.at - 1) as usize] = Some(r.runner);
            lob += 1;
        }
    }
    end_pa_lob(st, PaResult::BattedBall, Some(lob), ev);
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
    let PlayState::Live(lp) = &mut st.play else { return };
    ev.push(Event::RulesWarning { code: "play_timeout" });
    if lp.fair == FairFoul::Pending {
        lp.fair = FairFoul::Fair;
        ev.push(Event::FairFoulDecided { fair: true });
        ev.push(Event::PitchResult { call: PitchCall::InPlay, balls: st.balls, strikes: st.strikes });
        judge_pending_passes(st, ev);
    }
    let PlayState::Live(mut lp) = st.play.clone() else { return };
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
