//! Print game digests for a few seeds (behavior-preservation checks across refactors).
fn main() {
    for seed in [1u64, 2, 3, 42] {
        let g = bb_game::game::simulate_game(seed);
        let hex: String = g.digest.iter().map(|b| format!("{b:02x}")).collect();
        println!("{seed} {hex}");
    }
}
