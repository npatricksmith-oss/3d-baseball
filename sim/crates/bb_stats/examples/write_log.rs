//! Write a seed's finished event log to a file: `write_log <seed> <path>` (golden fixtures).
fn main() { let a: Vec<String> = std::env::args().collect(); std::fs::write(&a[2], bb_game::game::simulate_game(a[1].parse().unwrap()).log).unwrap(); }
