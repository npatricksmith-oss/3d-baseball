//! Diagnostic: one fielder's read of one play, every 0.2 s while the ball is on its first
//! flight (`live::read_trace`): where he is, how fast, where his latest read has the ball
//! coming down and how far that is from where it does. `-- <seed> <play> <fielder 1-9>`
use bb_game::live::read_trace;
use bb_math::Vec3;
fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let (seed, play, who) = (a[0], a[1] as u32, a[2] as usize - 1);
    read_trace::ON.with(|x| x.set(true));
    let mut g = bb_game::game::Game::new(seed, false);
    loop {
        read_trace::take();
        let Some(step) = g.step_pitch() else { break };
        if step.play != play {
            continue;
        }
        let recs = read_trace::take();
        let path = step.batted.as_ref().unwrap();
        let (tl, lx, ly) = path.first_bounce().unwrap();
        let land = Vec3::new(lx, ly, 0.0);
        println!("lands at {tl:.2} s ({lx:.1}, {ly:.1})");
        let mut next = 0.0;
        for r in recs.iter().filter(|r| r.i == who) {
            if r.t + 1e-9 < next {
                continue;
            }
            next = r.t + 0.2;
            let (b, _) = path.at(r.t);
            let bl = r.land.map(|(t, x, y)| format!("({x:6.1},{y:6.1}) at {t:.2}, off {:5.1} m (along {:+5.1})", (Vec3::new(x, y, 0.0) - land).norm(), (Vec3::new(x, y, 0.0) - land).dot(land.normalized()))).unwrap_or_default();
            println!("t {:.2} (to land {:.2}) ball z {:5.1} | pos ({:6.1},{:6.1}) v {:.1} role {} read {:.2} ago | belief {bl} | target ({:.1},{:.1})", r.t, tl - r.t, b.z, r.pos.x, r.pos.y, r.vel.norm(), r.role, r.t - r.read_at, r.target.x, r.target.y);
        }
        break;
    }
}
