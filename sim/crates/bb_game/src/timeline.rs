//! PlayTimeline encoding (data-formats §5): what a client needs to play back one pitch.
//! M4 skeleton: ball track (Hermite knots), markers, telemetry, and the view state after
//! the play. Bat, actor, and belief tracks come with the full fielding model.

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

/// Encode one pitch's timeline as a finished `BBTL` FlatBuffer.
pub fn encode(step: &PitchStep, st: &RulesState) -> Vec<u8> {
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
        let kv = fbb.create_vector(&knots);
        let spin = v3f(rec.release.w);
        segs.push(tl::BallSegment::create(&mut fbb, &tl::BallSegmentArgs { mode: BallMode::Flight, holder: 0, knots: Some(kv), spin_radps: Some(&spin) }));
    }
    if let Some(path) = &step.batted {
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

    // Markers.
    let mut mk: Vec<(f64, MarkerKind, Vec3)> = vec![(0.0, MarkerKind::Release, rec.release.r)];
    if let Some(p) = rec.plate {
        mk.push((0.42, MarkerKind::PlateCross, p));
    }
    if let Some((tc, b)) = rec.contact_state {
        mk.push((tc, MarkerKind::Contact, b.r));
    }
    if let Some(path) = &step.batted {
        mk.extend(path.bounces.iter().map(|&(t, x, y)| (t, MarkerKind::Bounce, Vec3::new(x, y, 0.0))));
        mk.extend(path.walls.iter().map(|&(t, x, y)| (t, MarkerKind::WallHit, Vec3::new(x, y, 0.0))));
    }
    let marks: Vec<_> = mk
        .into_iter()
        .map(|(t, kind, p)| {
            let pv = v3f(p);
            tl::Marker::create(&mut fbb, &tl::MarkerArgs { t: t as f32, kind, agent: 0, base: 0, p: Some(&pv) })
        })
        .collect();
    let markv = fbb.create_vector(&marks);

    // Telemetry (SI; the client converts for display).
    let spin = v3f(rec.release.w);
    let (ev_, la, spray) = rec.batted.unwrap_or((0.0, 0.0, 0.0));
    let plate = rec.plate.unwrap_or(Vec3::ZERO);
    let telem = tl::Telemetry::create(
        &mut fbb,
        &tl::TelemetryArgs {
            release_speed_mps: rec.release.v.norm() as f32,
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
    let duration = step.batted.as_ref().map(|p| p.t_end()).unwrap_or(step.pitch_flight.last().t) as f32;
    let root = tl::PlayTimeline::create(
        &mut fbb,
        &tl::PlayTimelineArgs {
            play_index: step.play,
            duration_s: duration,
            ball: Some(ball),
            markers: Some(markv),
            telemetry: Some(telem),
            view_after: Some(view),
            ..Default::default()
        },
    );
    fbb.finish(root, Some("BBTL"));
    fbb.finished_data().to_vec()
}
