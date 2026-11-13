# The overrun of first, and runner position samples (2026-11-13)

Coordinator item 3 (TASKS "Next"): the rules engine has judged the overrun of first
(OBR 5.09(b)(4)) and passing between bases from `RunnerPosition` samples since the
rules-passing work (rules-spec §5.5), but the live sim sent no samples, and its batter-runner
stopped dead on the bag. So in real games the protection never started and passing was
judged from base touches alone. Nothing here was tuned.

## The change

- **Samples** (`Sim::sample_runners`, rules-spec §2): every other 60 Hz tick (30 Hz,
  `SAMPLE_EVERY_TICKS`), after the runners move, each runner still on the base paths goes to
  the rules engine at his recorded position, all at the tick's time.
- **The overrun** (`Sim::step_overrun`, agent-models §6.2): a batter-runner who decides at
  first to stop there touches it, leaves it on the next tick, and runs straight down the
  first-base line, braking from his crossing speed at `OVERRUN_DECEL` = 4 m/s² [E] (8.4 m,
  28 ft, from 8.2 m/s); then he walks back at `OVERRUN_RETURN_MPS` = 2.5 m/s [E] and touches
  it again. The fielders make no play on him (he makes no turn); the play settles only once
  he is back.

The values are first guesses from watching batters run through the bag, not measurements.
They change only presentation and play length: he is protected the whole way, and the
defense ignores him.

## What it does in games

`cargo run --release -p bb_game --example overrun_probe -- 512 0` (every batted ball that
ended a plate appearance, seeds 0–511):

| Batter-runner touched first (of 30,614 batted balls) | 19,826 |
| :--- | ---: |
| ran through, walked back, touched it again | 8,421 |
| left it for second or beyond | 2,706 |
| left it, and the play ended while he was off it | 8,645 |
| &nbsp;&nbsp;he was put out (a fly caught while he ran through) | 7,689 |
| &nbsp;&nbsp;the third out made elsewhere, or the game ended | 121 |
| &nbsp;&nbsp;the ball left the park | 835 |
| never left it: put out later / not out | 37 / 17 |
| anything else | 0 |

Every batter-runner who left first and was off it when the play ended is accounted for:
put out (a fly caught while he was running through), the third out made elsewhere or the
game ending, or the ball leaving the park (a home run's awards end the play at once).

- **Passing outs from samples: none.** With samples in every play, no runner's sampled path
  progress passed another's by more than `pass_margin_m`. (From base touches alone there
  were none in 2,560 games either, rules-spec §5.5.)
- **Rules warnings: none.**
- **Run environment, 1,024 games:** seed 5: 5.36 → 5.39 runs/team-game; seed 2: 5.21 → 5.23
  (both changes in this session; the noise on a 1,024-game set is about ±0.07). Batted-ball
  hit rates are unchanged to the third decimal except LD on seed 2 (.710 → .709).
- **Unit tests:** with samples, a tag on the batter-runner walking back is no out, he is
  left on first, and he pulls up v²/(2·`OVERRUN_DECEL`) past the bag
  (`batter_runner_overruns_first_and_is_not_liable_coming_back`); the same play without
  samples retires him, so the samples are what protect him
  (`without_position_samples_the_overrunner_is_an_ordinary_runner`).

## Found on the way: swings that started before the decision

The changed game paths made the FFI test `nothing_is_given_away_before_the_decision_or_the_call`
fail on seed 11 play 227: the bat left the stance 7 ms before the batter's commit. The
swing is planned from the believed arrival, and execution noise and the timing adjustment
move it; nothing kept its start after the commit. Over 64 games, 13 of 8,199 swings (0.16%)
started before the decision, by up to 18 ms (Codex: 12 of 8,172 at the commit before this
work, on its own game paths). A swing now starts no earlier than the commit,
moved whole, so it meets the ball that much later (`Swing::start_no_earlier_than`,
agent-models §3.4); after it, none do. Test: `no_swing_starts_before_the_batter_decides`.

## Not done

- A voluntary turn toward second on a hit and back (he decides at the bag to go on or run
  through). A batter-runner sent back because the runner ahead holds second does already
  retreat to first, with the attempt judged from the samples (Codex review #1); none did
  in the 512 games above. Also not done: a slide into first; a batter-runner
  who sees an overthrow during his overrun and goes for second (he decides again only once
  back on the bag, and only if he was waiting on a runner ahead).
- The play cannot end until he is back: about 6.2 s after he crosses the bag from 8.2 m/s
  (2.0 s braking, 4.2 s walking back), so plays that put him on first run longer than before
  (not measured). The client plays the track back as recorded; it has not been watched.
