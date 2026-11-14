//! A home run on screen (A16): after the ball clears the fence, the timeline has the batter and
//! every runner trot round to home, touching each base still ahead of them in order (OBR 5.06(b)(4)(A):
//! the batter and runners must touch the bases in order on a home run), at a trot, and the clip
//! lasts until the last of them scores. Checked against the rules and the timeline's own tracks.

use bb_game::game::Game;
use bb_rules::{Event, PaResult};

const H: f32 = 19.397_3; // 27.432 / √2: first base is at (H, H), second (0, 2H), third (−H, H)

#[test]
fn home_run_trots_touch_every_base_ahead_and_end_at_home() {
    let bases = [(H, H), (0.0, 2.0 * H), (-H, H), (0.0, 0.0)];
    let (mut homers, mut runners) = (0, 0);
    for seed in 0..10u64 {
        let mut g = Game::new(seed, true);
        while let Some(s) = g.step_pitch() {
            if !s.rules_events.iter().any(|e| matches!(e, Event::PlateAppearanceEnded { result: PaResult::HomeRun, .. })) {
                continue;
            }
            homers += 1;
            let bytes = bb_game::timeline::encode(&s, &g.st, &g.rosters);
            let tl = flatbuffers::root::<bb_schema::tl::PlayTimeline>(&bytes).unwrap();
            let dead = tl.markers().unwrap().iter().find(|m| m.kind() == bb_schema::tl::MarkerKind::BallDead).map(|m| m.t()).expect("a home run's BallDead marker");
            // Everyone who scored: the batter and the runners on base (RunScored).
            let scorers: Vec<u16> = s.rules_events.iter().filter_map(|e| if let Event::RunScored { runner } = e { Some(*runner) } else { None }).collect();
            for track in tl.actors().unwrap().iter().filter(|a| scorers.contains(&a.agent())) {
                runners += 1;
                let knots: Vec<_> = track.root().unwrap().iter().filter(|k| k.t() >= dead).map(|k| (k.t(), k.p().x(), k.p().y())).collect();
                let (t_last, x, y) = *knots.last().unwrap();
                assert!(x.hypot(y) < 0.3, "seed {seed} play {}: {:#x} ends at ({x:.2}, {y:.2}), not home", s.play, track.agent());
                assert!(t_last <= tl.duration_s() + 1e-3, "the clip covers the trot");
                // The bases he passes after the ball is gone, in order, ending at home: each
                // touched (within 0.3 m of its centre) after the one before.
                let mut next = 0usize;
                let first = knots[0];
                // Bases already behind him: those nearer home along the path than where he starts.
                let along = |x: f32, y: f32| -> f32 {
                    // Progress round the bases, 0..4, from the nearest leg.
                    let mut best = (f32::INFINITY, 0.0);
                    let pts = [(0.0f32, 0.0f32), bases[0], bases[1], bases[2], bases[3]];
                    for k in 0..4 {
                        let (ax, ay, bx, by) = (pts[k].0, pts[k].1, pts[k + 1].0, pts[k + 1].1);
                        let (dx, dy) = (bx - ax, by - ay);
                        let u = (((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
                        let d = (x - ax - dx * u).hypot(y - ay - dy * u);
                        if d < best.0 {
                            best = (d, k as f32 + u);
                        }
                    }
                    best.1
                };
                let start = along(first.1, first.2);
                while next < 4 && (next as f32 + 1.0) <= start + 0.02 {
                    next += 1;
                }
                let mut t_touch = Vec::new();
                for &(t, x, y) in &knots {
                    if next < 4 && (x - bases[next].0).hypot(y - bases[next].1) < 0.3 {
                        t_touch.push(t);
                        next += 1;
                    }
                }
                assert_eq!(next, 4, "seed {seed} play {}: {:#x} from progress {start:.2} missed a base", s.play, track.agent());
                // At a trot: from his first base touch to home, 4.5–5 m/s.
                if t_touch.len() >= 2 {
                    let legs = (t_touch.len() - 1) as f32 * 27.432;
                    let v = legs / (t_touch.last().unwrap() - t_touch[0]);
                    assert!((4.5..=5.0).contains(&v), "seed {seed} play {}: trot {v:.2} m/s", s.play);
                }
            }
        }
    }
    assert!(homers >= 5 && runners >= homers, "{homers} home runs, {runners} trots");
}
