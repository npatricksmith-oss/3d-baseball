//! Profile one game's live-ball costs.
use bb_game::live::prof;
fn main() {
    let t0 = std::time::Instant::now();
    let g = bb_game::game::simulate_game(3);
    let total = t0.elapsed().as_secs_f64() * 1e3;
    let (pn, pc) = prof::PRED.with(|c| c.get());
    let (inn, ic) = prof::INTER.with(|c| c.get());
    let (rn, rc) = prof::RUNNER.with(|c| c.get());
    let (ln, _) = prof::LIVE.with(|c| c.get());
    let ticks = prof::TICKS.with(|c| c.get());
    println!("game {total:.0} ms (live plays {:.0} ms), {} pitches, {} BIP; predict {:.0} ms in {pc} calls; intercept {:.0} ms in {ic} calls; runner reads {:.0} ms in {rc} calls; {ticks} ticks", ln as f64 / 1e6, g.pitches, g.bip, pn as f64 / 1e6, inn as f64 / 1e6, rn as f64 / 1e6);
}
