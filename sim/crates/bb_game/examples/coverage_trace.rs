//! Coverage trace over games with literal seeds 0..n (Codex fielder-communication recheck P1-2):
//! run with `BB_COVER_TRACE=1`; stdout gets each game's digest, stderr one line per change of
//! the defense's jobs. Analyse with `python3 analysis/coverage_gaps.py <stderr log>`.
//!
//! `BB_COVER_TRACE=1 cargo run --release -p bb_game --example coverage_trace -- 50 2> trace.log`
fn main() {
    let n: u64 = std::env::args().nth(1).unwrap_or("50".into()).parse().unwrap();
    for seed in 0..n {
        let g = bb_game::game::simulate_game(seed);
        let digest: String = g.digest.iter().map(|b| format!("{b:02x}")).collect();
        println!("{seed} {digest}");
    }
}
