//! Diagnostic: one play's fielder tracks. `-- <seed> <play>`
fn main() {
    let a: Vec<u64> = std::env::args().skip(1).map(|s| s.parse().unwrap()).collect();
    let (seed, play) = (a[0], a[1] as u32);
    let mut g = bb_game::game::Game::new(seed, true);
    while let Some(step) = g.step_pitch() {
        if step.play != play {
            continue;
        }
        let path = step.batted.as_ref().unwrap();
        println!("first bounce {:?}", path.first_bounce());
        let v = &step.viz;
        for (t, p) in v.batted.iter().step_by(15).map(|s| (s[0], [s[1], s[2], s[3]])) {
            println!("ball t {t:.2} ({:.1},{:.1},{:.1})", p[0], p[1], p[2]);
        }
        for (id, tr) in &v.actors {
            if id & 0x80 == 0 { continue; }
            let s: Vec<String> = tr.iter().step_by(3).map(|(t, p)| format!("{t:.1}:({:.0},{:.0})", p.x, p.y)).collect();
            println!("f{} {}", id & 0x7F, s.join(" "));
        }
        println!("secured {:?}", v.secured);
        println!("released {:?}", v.released);
        for f in &step.facts { println!("  {f:?}"); }
        break;
    }
}
