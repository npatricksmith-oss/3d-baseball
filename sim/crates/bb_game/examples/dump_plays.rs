//! Dump a game's balls in play (ball, fielders, runners) as JSON for plotting.
fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(7);
    let g = bb_game::game::simulate_game_with(seed, true);
    let mut out = String::from("[");
    let mut first = true;
    for v in g.viz.iter().filter(|v| !v.batted.is_empty()).take(12) {
        if !first {
            out.push(',');
        }
        first = false;
        let ball: Vec<String> = v.batted.iter().map(|s| format!("[{:.2},{:.2},{:.2}]", s[1], s[2], s[3])).collect();
        let actors: Vec<String> = v
            .actors
            .iter()
            .map(|(id, s)| format!("{{\"id\":{},\"p\":[{}]}}", id, s.iter().map(|(_, p)| format!("[{:.2},{:.2}]", p.x, p.y)).collect::<Vec<_>>().join(",")))
            .collect();
        out += &format!("{{\"label\":{:?},\"ball\":[{}],\"actors\":[{}]}}", v.label, ball.join(","), actors.join(","));
    }
    out.push(']');
    println!("{out}");
}
