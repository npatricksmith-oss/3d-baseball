//! PlayTimeline encoding (data-formats §5): what a client needs to play back one pitch.
//! M4 skeleton: ball track (Hermite knots), actor root tracks, markers, telemetry, and the
//! view state after the play. Bat and belief tracks come later.
//!
//! When the game records visualization data (`Game::new(seed, true)`), the ball track after
//! contact is the **realized** ball (fielded, held, thrown) and every fielder and runner has a
//! root track. Without it, only the batted ball's free path is known and there are no actors.

use bb_math::Vec3;
use bb_physics::ground::Segment;
use bb_rules::{Half, RulesState};
use bb_schema::tl::{self, BallKnot, BallMode, MarkerKind};
use bb_schema::{Vec3f, ss};
use flatbuffers::FlatBufferBuilder;

use crate::game::PitchStep;

fn v3f(v: Vec3) -> Vec3f {
    Vec3f::new(v.x as f32, v.y as f32, v.z as f32)
}

/// Umpire signal latency, s: how long after a judgment the players have it (rules-spec §1.3,
/// `T_signal`).
pub const UMPIRE_SIGNAL_S: f64 = 0.2;
/// Presentation only [E]: after the call of ball four or a hit batter, how long before the
/// batter and forced runners start for their bases, s.
pub const AWARD_REACTION_S: f64 = 0.4;
/// Presentation only [E]: how long before the swing starts the bat leaves the stance, s. At
/// most the batter's decision-to-motor latency (0.05 s), so it never begins before he has
/// committed.
pub const LOAD_S: f64 = 0.05;

/// The bat in the stance, for a batter standing at `at`: knob and axis (knob → tip), the bat
/// upright at the back shoulder and tipped a little back toward the catcher.
pub fn stance_pose(at: Vec3) -> (Vec3, Vec3) {
    let back = if at.x < 0.0 { -1.0 } else { 1.0 };
    (at + Vec3::new(0.0, -0.15, 1.25), Vec3::new(0.15 * back, -0.25, 1.0).normalized())
}

/// Encode one pitch's timeline as a finished `BBTL` FlatBuffer.
pub fn encode(step: &PitchStep, st: &RulesState, rosters: &crate::roster::Rosters) -> Vec<u8> {
    let mut fbb = FlatBufferBuilder::with_capacity(16 * 1024);
    let rec = &step.record;
    // Ball track: the pitch, then each batted-ball segment.
    let mut segs = Vec::new();
    let t_contact = rec.contact_state.map(|c| c.0);
    {
        let knots: Vec<BallKnot> = step
            .pitch_flight
            .knots
            .iter()
            .filter(|k| t_contact.map(|tc| k.t <= tc).unwrap_or(true))
            .map(|k| BallKnot::new(k.t as f32, &v3f(k.s.r), &v3f(k.s.v)))
            .collect();
        // End the pitch exactly at contact, so the batted ball starts where the pitch stops.
        let mut knots = knots;
        if let Some(tc) = t_contact {
            let (r, v) = step.pitch_flight.at(tc);
            knots.push(BallKnot::new(tc as f32, &v3f(r), &v3f(v)));
        }
        let kv = fbb.create_vector(&knots);
        let spin = v3f(rec.release.w);
        segs.push(tl::BallSegment::create(&mut fbb, &tl::BallSegmentArgs { mode: BallMode::Flight, holder: 0, knots: Some(kv), spin_radps: Some(&spin) }));
    }
    let realized = &step.viz.batted;
    if realized.len() >= 2 {
        // The ball as it actually went: sampled positions. Each knot's velocity is the
        // smaller of the slopes on its two sides, per axis, and zero where they disagree
        // (minmod), so the Hermite curve never overshoots a sample: a grounder does not dip
        // under the turf before a pickup, and a caught ball stops in the glove.
        let n = realized.len();
        let p = |i: usize| Vec3::new(realized[i][1], realized[i][2], realized[i][3]);
        let slope = |i: usize| (p(i + 1) - p(i)) / (realized[i + 1][0] - realized[i][0]).max(1e-6);
        let minmod = |a: f64, b: f64| if a * b <= 0.0 { 0.0 } else if a.abs() < b.abs() { a } else { b };
        let knots: Vec<BallKnot> = (0..n)
            .map(|i| {
                let v = if i == 0 {
                    slope(0)
                } else if i == n - 1 {
                    slope(n - 2)
                } else {
                    let (a, b) = (slope(i - 1), slope(i));
                    Vec3::new(minmod(a.x, b.x), minmod(a.y, b.y), minmod(a.z, b.z))
                };
                BallKnot::new(realized[i][0] as f32, &v3f(p(i)), &v3f(v))
            })
            .collect();
        let kv = fbb.create_vector(&knots);
        segs.push(tl::BallSegment::create(&mut fbb, &tl::BallSegmentArgs { mode: BallMode::Flight, holder: 0, knots: Some(kv), spin_radps: None }));
    } else if let Some(path) = &step.batted {
        for seg in &path.segments {
            let (mode, knots, spin): (BallMode, Vec<BallKnot>, Vec3) = match seg {
                Segment::Flight { tr, .. } => (BallMode::Flight, tr.knots.iter().map(|k| BallKnot::new(k.t as f32, &v3f(k.s.r), &v3f(k.s.v))).collect(), tr.first().s.w),
                Segment::Roll { t0, t_stop, .. } => {
                    let mut ks = Vec::new();
                    let mut t = *t0;
                    while t < *t_stop {
                        let (r, v) = path.at(t);
                        ks.push(BallKnot::new(t as f32, &v3f(r), &v3f(v)));
                        t += 0.05;
                    }
                    let (r, v) = path.at(*t_stop);
                    ks.push(BallKnot::new(*t_stop as f32, &v3f(r), &v3f(v)));
                    (BallMode::Rolling, ks, Vec3::ZERO)
                }
            };
            let kv = fbb.create_vector(&knots);
            let sp = v3f(spin);
            segs.push(tl::BallSegment::create(&mut fbb, &tl::BallSegmentArgs { mode, holder: 0, knots: Some(kv), spin_radps: Some(&sp) }));
        }
    }
    let segv = fbb.create_vector(&segs);
    let ball = tl::BallTrack::create(&mut fbb, &tl::BallTrackArgs { segments: Some(segv) });

    // Actors: a root track for everyone on the field. Those who moved during a live ball
    // have their sampled track, preceded by where they stood until it starts (the batter in
    // his box until contact); everyone else stands where he was at the pitch.
    let mut tracks: Vec<(u16, Vec<(f64, Vec3)>)> = step
        .viz
        .standing
        .iter()
        .map(|(id, at)| {
            let mut v = vec![(0.0, *at)];
            if let Some(s) = step.viz.actors.iter().find(|(a, s)| a == id && !s.is_empty()).map(|x| &x.1) {
                v.push(((s[0].0 - 0.02).max(0.0), *at));
                // The realized positions, as the live simulator had them (the batter's run out
                // of his box included: `live::Sim::runner_pos`).
                v.extend(s.iter().copied());
            }
            (*id, v)
        })
        .collect();
    let mut t_end = match realized.last() {
        Some(k) if realized.len() >= 2 => k[0],
        _ => step.batted.as_ref().map(|p| p.t_end()).unwrap_or(step.pitch_flight.last().t),
    };
    t_end = tracks.iter().filter_map(|(_, v)| v.last().map(|k| k.0)).fold(t_end, f64::max);
    // Bases awarded with the ball dead (a walk, a hit batter, a home run, a ball out of
    // play): the runners jog to them after the play. This is presentation only: the rules
    // have already put them there, and nothing here feeds back into the game.
    const JOG_MPS: f64 = 6.0;
    // A home run is a trot, not a jog to a base: 109.7 m in about 23 s, a typical major-league
    // home-run trot (A16; the client's jog clip reads 6 m/s as a run).
    const HR_TROT_MPS: f64 = 4.8;
    let home_run = step.rules_events.iter().any(|e| matches!(e, bb_rules::Event::PlateAppearanceEnded { result: bb_rules::PaResult::HomeRun, .. }));
    let pace = if home_run { HR_TROT_MPS } else { JOG_MPS };
    const BASE_L: f64 = 27.432;
    // On ball four or a hit batter nobody moves until the umpire's call: it comes when the
    // catcher has the ball (the pitch's end), reaches the players after the signal latency
    // (rules-spec §1.3), and they take a moment to react (drop the bat, look, go).
    // (A home run or a ball out of play is already over by `t_end`: they go 0.4 s after.)
    let t_call = step.pitch_flight.last().t + UMPIRE_SIGNAL_S;
    let t_after = t_end + 0.4;
    for e in &step.rules_events {
        let bb_rules::Event::RunnerAdvanced { runner, from, to, cause } = e else { continue };
        if *cause == bb_rules::AdvanceCause::OnPlay || to <= from {
            continue;
        }
        let t_jog = match cause {
            bb_rules::AdvanceCause::Walk | bb_rules::AdvanceCause::HitByPitch => t_after.max(t_call + AWARD_REACTION_S),
            _ => t_after,
        };
        let Some((_, track)) = tracks.iter_mut().find(|(id, _)| id == runner) else { continue };
        let (s_lo, s_hi) = (*from as f64 * BASE_L, *to as f64 * BASE_L);
        // Where he already is on the base path (a batter may be past first when the ball
        // clears the fence, a runner on a walk is off the base with his lead): the nearest
        // point of his awarded stretch.
        let here = track.last().unwrap().1;
        let mut s0 = s_lo;
        // (A runner who never moved is at his lead, not on the base: start from there too.)
        {
            let mut best = f64::INFINITY;
            let mut s = s_lo;
            while s <= s_hi {
                let d = (crate::live::path_pos(s) - here).norm();
                if d < best {
                    (best, s0) = (d, s);
                }
                s += 0.25;
            }
        }
        // He stands where he is until he goes (root tracks interpolate linearly, so without
        // this knot he would drift toward the base path from the moment of release), then
        // steps onto the path and jogs along it.
        track.push((t_jog, here));
        let t_on = t_jog + (crate::live::path_pos(s0) - here).norm() / pace;
        track.push((t_on, crate::live::path_pos(s0)));
        let mut k = 1;
        loop {
            let s = (s0 + pace * 0.1 * k as f64).min(s_hi);
            track.push((t_on + 0.1 * k as f64, crate::live::path_pos(s)));
            if s >= s_hi {
                break;
            }
            k += 1;
        }
        t_end = t_end.max(track.last().unwrap().0);
    }
    // A dive the play ended during (the deciding fact can come on the dive's own tick): the
    // diver finishes his lunge to where his body lands and lies there until he is up, and
    // the clip lasts until then. Presentation only, like the jogs above.
    for &(t, id, _, land) in &step.viz.dives {
        let up = t + crate::live::DIVE_GETUP_S;
        let Some((_, track)) = tracks.iter_mut().find(|(a, _)| *a == id) else { continue };
        let last = track.last().unwrap().0;
        if last < up {
            if last < t + crate::live::DIVE_LUNGE_S {
                track.push((t + crate::live::DIVE_LUNGE_S, land));
            }
            track.push((up, land));
            t_end = t_end.max(up);
        }
    }
    // The play is not over for a viewer until the umpire has signalled its last out.
    t_end = step.calls.iter().map(|c| c.t_call()).fold(t_end, f64::max);
    let mut actor_offs = Vec::new();
    for (id, s) in &tracks {
        let knots: Vec<tl::RootKnot> = (0..s.len())
            .map(|i| {
                let (a, b) = (i.saturating_sub(1), (i + 1).min(s.len() - 1));
                // A knot that begins or ends a hold (the same spot as a neighbour) is at rest:
                // its velocity and facing must not look ahead to where he goes next (Codex
                // batter-presentation review #3).
                let holds = (a < i && s[a].1 == s[i].1) || (b > i && s[b].1 == s[i].1);
                let v = if b > a && !holds { (s[b].1 - s[a].1) / (s[b].0 - s[a].0).max(1e-6) } else { Vec3::ZERO };
                let yaw = if v.norm() > 0.5 { bb_math::m::atan2(v.x, v.y) } else { 0.0 };
                tl::RootKnot::new(s[i].0 as f32, &v3f(s[i].1), &v3f(v), yaw as f32)
            })
            .collect();
        let kv = fbb.create_vector(&knots);
        actor_offs.push(tl::ActorTrack::create(&mut fbb, &tl::ActorTrackArgs { agent: *id, root: Some(kv), ..Default::default() }));
    }
    let actors = if actor_offs.is_empty() { None } else { Some(fbb.create_vector(&actor_offs)) };

    // Bat: every pitch starts from the same stance, the bat upright at the batter's back
    // shoulder, so nothing before the swing tells a viewer whether he will swing. A swing
    // leaves the stance at most `LOAD_S` before it starts and never before his decision
    // (agent-models §3.6), blending into the swing's own start pose, then follows the
    // executed swing every 5 ms through the follow-through.
    let quat_from_x = |a: Vec3| {
        // Shortest rotation taking +X to the unit vector `a` (bat axis = rot · +X).
        let (c, w) = (Vec3::new(0.0, -a.z, a.y), 1.0 + a.x);
        let q = if w < 1e-6 { (0.0, 0.0, 1.0, 0.0) } else { (c.x, c.y, c.z, w) };
        let n = bb_math::m::sqrt(q.0 * q.0 + q.1 * q.1 + q.2 * q.2 + q.3 * q.3);
        bb_schema::Quatf::new((q.0 / n) as f32, (q.1 / n) as f32, (q.2 / n) as f32, (q.3 / n) as f32)
    };
    let stance = step.viz.standing.last().map(|(_, at)| stance_pose(*at));
    let mut bat_samples: Vec<tl::BatSample> = stance.iter().map(|(knob, axis)| tl::BatSample::new(0.0, &v3f(*knob), &quat_from_x(*axis))).collect();
    if let Some(sw) = &rec.swing {
        use bb_physics::contact::BatPose;
        let (t0, t1) = (sw.t_start(), sw.t_c + 0.12);
        if t0 <= 0.0 {
            // A swing under way at the release has no stance to start from.
            bat_samples.clear();
        } else if let Some((knob_s, axis_s)) = stance {
            // Never before his decision (Codex batter-presentation review #1): the swing's
            // start can come less than the latency after it, through timing noise and the
            // in-flight aim adjustment, and then the load is shorter. The first load sample
            // is the stance itself.
            let t_load = (t0 - LOAD_S).max(rec.t_decide);
            let start = sw.at(t0);
            let mut t = t_load;
            while t < t0 {
                let u = (t - t_load) / (t0 - t_load).max(1e-6);
                let knob = knob_s + (start.knob - knob_s) * u;
                let a = axis_s * (1.0 - u) + start.axis * u;
                let axis = if a.norm() > 1e-6 { a.normalized() } else { start.axis };
                bat_samples.push(tl::BatSample::new(t as f32, &v3f(knob), &quat_from_x(axis)));
                t += 0.005;
            }
        }
        let n = ((t1 - t0) / 0.005).ceil() as usize;
        bat_samples.extend((0..=n).map(|i| {
            let t = t0 + (t1 - t0) * i as f64 / n as f64;
            let k = sw.at(t);
            tl::BatSample::new(t as f32, &v3f(k.knob), &quat_from_x(k.axis))
        }));
    }
    let bat = if bat_samples.is_empty() {
        None
    } else {
        let sv = fbb.create_vector(&bat_samples);
        Some(tl::BatTrack::create(&mut fbb, &tl::BatTrackArgs { agent: st.batter, model_id: 0, samples: Some(sv) }))
    };

    // Markers.
    let mut mk: Vec<(f64, MarkerKind, Vec3)> = vec![(0.0, MarkerKind::Release, rec.release.r)];
    if let Some(p) = rec.plate {
        mk.push((0.42, MarkerKind::PlateCross, p));
    }
    if let Some(sw) = &rec.swing {
        use bb_physics::contact::BatPose;
        mk.push((sw.t_start(), MarkerKind::SwingStart, sw.at(sw.t_start()).knob));
    }
    if let Some((tc, b)) = rec.contact_state {
        mk.push((tc, MarkerKind::Contact, b.r));
    }
    if let Some(path) = &step.batted {
        mk.extend(path.bounces.iter().map(|&(t, x, y)| (t, MarkerKind::Bounce, Vec3::new(x, y, 0.0))));
        mk.extend(path.walls.iter().map(|&(t, x, y)| (t, MarkerKind::WallHit, Vec3::new(x, y, 0.0))));
    }
    // Possession: who secured the ball and when, and each throw's release, by agent.
    let mut mk: Vec<(f64, MarkerKind, Vec3, u16)> = mk.into_iter().map(|(t, k, p)| (t, k, p, 0)).collect();
    mk.extend(step.viz.secured.iter().map(|&(t, id)| (t, MarkerKind::Catch, Vec3::ZERO, id)));
    mk.extend(step.viz.released.iter().map(|&(t, id)| (t, MarkerKind::ThrowRelease, Vec3::ZERO, id)));
    // Dives: the fielder leaves his feet (p = where) and is back on them (p = where he landed).
    mk.extend(step.viz.dives.iter().map(|&(t, id, from, _)| (t, MarkerKind::Dive, from, id)));
    mk.extend(step.viz.dives.iter().map(|&(t, id, _, land)| (t + crate::live::DIVE_GETUP_S, MarkerKind::DiveUp, land, id)));
    let mut marks: Vec<_> = mk
        .into_iter()
        .map(|(t, kind, p, agent)| {
            let pv = v3f(p);
            tl::Marker::create(&mut fbb, &tl::MarkerArgs { t: t as f32, kind, agent, base: 0, p: Some(&pv), margin_s: 0.0 })
        })
        .collect();
    // The umpire's out calls (rules-spec §1.3), at the moment he signals each.
    for c in &step.calls {
        let pv = v3f(c.at);
        marks.push(tl::Marker::create(
            &mut fbb,
            &tl::MarkerArgs { t: c.t_call() as f32, kind: MarkerKind::UmpireCall, agent: c.runner, base: c.base, p: Some(&pv), margin_s: c.margin_s.min(f32::MAX as f64) as f32 },
        ));
    }
    // The ball left the playing field (rules-spec §5.8): when it was dead, the ground point
    // where it left, and why in `base` (the event log's `DeadBallReason`: 1 out of play, a
    // foul or a throw; 2 home run; 3 ground-rule double), so a client can cut back to the field.
    let reason = step.rules_events.iter().find_map(|e| match e {
        bb_rules::Event::BallDead { reason } => Some(match reason {
            bb_rules::DeadBall::OutOfPlay => 1u8,
            bb_rules::DeadBall::HomeRun => 2,
            bb_rules::DeadBall::GroundRuleDouble => 3,
        }),
        _ => None,
    });
    for f in &step.facts {
        if let bb_rules::Input::BallTouched { t, what: bb_rules::Touch::OutOfPlay { x, y, .. } } = *f {
            let pv = v3f(Vec3::new(x, y, 0.0));
            marks.push(tl::Marker::create(&mut fbb, &tl::MarkerArgs { t: t as f32, kind: MarkerKind::BallDead, agent: 0, base: reason.unwrap_or(0), p: Some(&pv), margin_s: 0.0 }));
        }
    }
    let markv = fbb.create_vector(&marks);

    // Telemetry (SI; the client converts for display).
    let spin = v3f(rec.release.w);
    let (ev_, la, spray) = rec.batted.unwrap_or((0.0, 0.0, 0.0));
    let plate = rec.plate.unwrap_or(Vec3::ZERO);
    let telem = tl::Telemetry::create(
        &mut fbb,
        &tl::TelemetryArgs {
            release_speed_mps: rec.release.v.norm() as f32,
            projected_distance_m: rec.projected_distance_m.unwrap_or(0.0) as f32,
            spin_radps: Some(&spin),
            plate_x_m: plate.x as f32,
            plate_z_m: plate.z as f32,
            exit_velocity_mps: ev_ as f32,
            launch_angle_rad: la as f32,
            spray_angle_rad: spray as f32,
            ..Default::default()
        },
    );

    // View state after the play.
    let runs = st.score();
    let away = ss::TeamView::create(&mut fbb, &ss::TeamViewArgs { runs: runs[0].min(255) as u8, ..Default::default() });
    let home = ss::TeamView::create(&mut fbb, &ss::TeamViewArgs { runs: runs[1].min(255) as u8, ..Default::default() });
    let base = |i: usize| st.bases[i].unwrap_or(0xFFFF);
    let view = ss::ViewState::create(
        &mut fbb,
        &ss::ViewStateArgs {
            inning: st.inning,
            top: st.half == Half::Top,
            outs: st.outs,
            balls: st.balls,
            strikes: st.strikes,
            runner_1b: base(0),
            runner_2b: base(1),
            runner_3b: base(2),
            batter: st.batter,
            away: Some(away),
            home: Some(home),
            ..Default::default()
        },
    );
    let duration = t_end as f32;
    let text = crate::describe::describe(rec.call, rec.batted.map(|b| b.1.to_degrees()), &step.rules_events, &step.facts, Some(rosters));
    let summary = fbb.create_string(&text);
    let root = tl::PlayTimeline::create(
        &mut fbb,
        &tl::PlayTimelineArgs {
            play_index: step.play,
            duration_s: duration,
            ball: Some(ball),
            bat,
            actors,
            summary: Some(summary),
            batter: step.batter,
            markers: Some(markv),
            telemetry: Some(telem),
            view_after: Some(view),
            ..Default::default()
        },
    );
    fbb.finish(root, Some("BBTL"));
    fbb.finished_data().to_vec()
}
