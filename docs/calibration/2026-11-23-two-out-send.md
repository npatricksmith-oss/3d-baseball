# The two-out send from second (Q38 (a)): fit and effect (2026-11-23)

Branch `work/two-out-caution`, from main `2a9e580`. The owner chose Q38 (a) on 2026-11-23: **the third-base coach's two-out pre-decision, "send unless the runner believes he is clearly beaten", with the caution set by outs and fitted to the league's two-out sends.** The diagnosis is in the [runner-start note §1](2026-11-23-runner-start-open.md#1-two-out-sends-from-second-on-singles). The rule is in agent-models §6.3, "The two-out send from second".

## 1. What was built

- `AdvanceParams::send_two_out_z` [E] = **−0.2** (`bb_agents::running`). With two out on the board as he sees them, a runner who started the play on second and decides on home uses it in place of `caution_z` (1.0). This covers both his decision at third (`runner_decide`) and his relook between third and home (`runner_relook`). `AdvanceParams::for_decision(start, next, outs)` picks the caution. The agents see no world state, and no new draw or RNG purpose is added.
- `Game::with_advance` (calibration only, like `with_runner_start`), and `BB_SEND_TWO_OUT_Z=<z>` in the `runner_timing` and `runner_rates` examples for the sweep. `runner_timing` now also counts runners put out on singles and the runner from first on doubles.

**Band and mechanism (calibration-targets §2.3, item 3; §2.4).** The band: Retrosheet 2024, two-out runners from second on outfield singles, by fielder: **91.3% sent from left, 96.2% from center, 92.9% from right; 5.6%, 0.6% and 2.2% of the sends put out at home** (runner-start note §1, recounted here with `retro_send2.py`). The corners pooled: 1,002 sent of 1,088 (92.1%), and 39 put out (3.9% of those sent). The mechanism shown to be off: the runner's caution. In the sim, the corner runners who held were all 0–0.27 s behind the ball alone, and the catcher's 0.25 s tag would have made most of them safe. Every other part of the race was already in line with its source (runner-start note §1, "Each leg"). Only one parameter changed, so there is no compensating pair.

**Scope: only the runner from second going home.**
- **The source.** The coaching rule is about this send: "with two out the coach decides before the pitch to send the runner from second home on a single" (Guilliams, decision-model note §1.6).
- **First to third with two out** is not touched. It already runs at 47.7% of all singles with a runner on first. Retrosheet 2024 has 42.5% (outfield singles with second open; `retro_other.py`; the sim's count also includes infield singles and second occupied).
- **First to home on a double with two out** is not touched either. It is already 92.3% (323/350), against Retrosheet's 58.9% (521/885). That gap is separate and is not caused by this caution. A first version that applied the two-out value to every runner deciding on home moved it by at most +1.7 points (to 94.0% at 0.1), within noise.
- **The batter-runner** never decides on home on a single.
- **With 0–1 out** the general caution stands.

**Why the value is below zero.** His read of each race has a 0.15 s timing error per look (`RUNNER_TIMING_SIGMA_S`). At 0.1 and 0.3, runners 0.1–0.27 s behind the ball alone still held on about half their reads, and the corners stopped at 89%. At −0.2 he goes when Φ(margin/σ + 0.2) beats the 41.5% break-even. That means he goes when he believes he is no more than about 0.08 s behind ball plus tag, which is about 0.33 s behind the ball alone. Before, he went only when he believed he was 0.16 s ahead of ball plus tag. This is the coach's "send unless clearly beaten".

## 2. The sweep

Seeds 0–299 and 1000–1299 direct, average fielders, `runner_timing` (`sim/target/codex-probes/q38/`, outside the tracked files: `sweep.sh`, `timing_*.csv/.err`, `scripts/an_q38.py`, `scripts/table.py`, and the Retrosheet scripts `scripts/retro_send2.py` and `scripts/retro_other.py`). Chances are scorer-credited outfield singles on which the runner from second reached third with that base open (the runner-start note's "eligible"). "Out" counts every runner put out on the play after passing third. At 1.0 the games are main's: the probe's seed-0 digest matched the standard game.

| `send_two_out_z` | Left, two out | Center | Right | Corners: sent, of sends out | 0–1 out, left / center / right | First to third, 0 / 1 / 2 out | Second to home, two out | First to home on a double, two out |
|---|---|---|---|---|---|---|---|---|
| 1.0 (before) | 72/83 (86.7%), 0 out | 119/121 (98.3%), 0 out | 74/93 (79.6%), 2 out | 83.0%, 1.4% | 52.7% / 72.6% / 48.8% | 214/569, 279/892, 386/809 | 296/394 (75.1%) | 323/350, 4 out |
| 0.5 † | 71/81 (87.7%), 0 out | 116/118 (98.3%), 0 out | 79/93 (84.9%), 3 out | 86.2%, 2.0% | 52.3% / 72.4% / 48.8% | 214/570, 280/894, 378/802 | 298/387 (77.0%) | 327/350, 3 out |
| 0.3 † | 72/80 (90.0%), 0 out | 116/118 (98.3%), 0 out | 82/93 (88.2%), 4 out | 89.0%, 2.6% | 53.1% / 72.0% / 48.8% | 213/567, 281/905, 378/803 | 301/386 (78.0%) | 324/347, 4 out |
| 0.1 † | 71/79 (89.9%), 0 out | 119/120 (99.2%), 0 out | 82/92 (89.1%), 4 out | 89.5%, 2.6% | 53.9% / 71.8% / 49.4% | 214/568, 281/907, 376/803 | 303/385 (78.7%) | 328/349, 4 out |
| **−0.2 (built)** | 77/82 (93.9%), 0 out | 120/121 (99.2%), 0 out | 86/93 (92.5%), 7 out | **93.1%, 4.3%** | 53.2% / 72.1% / 49.1% | 213/566, 280/891, 384/807 | 308/389 (79.2%) | 322/348, 3 out |
| −0.3 | 78/82 (95.1%), 1 out | 120/121 (99.2%), 0 out | 86/93 (92.5%), 7 out | 93.7%, 4.9% | 53.2% / 72.1% / 49.1% | 214/567, 283/898, 383/805 | 308/388 (79.4%) | 322/348, 3 out |
| −0.6 | 81/82 (98.8%), 2 out | 121/121 (100.0%), 0 out | 88/93 (94.6%), 9 out | 96.6%, 6.5% | 52.7% / 72.1% / 49.1% | 215/569, 284/901, 382/805 | 311/388 (80.2%) | 320/346, 3 out |
| −1.0 | 81/81 (100.0%), 3 out | 122/122 (100.0%), 0 out | 88/93 (94.6%), 9 out | 97.1%, 7.1% | 52.7% / 72.2% / 48.8% | 215/569, 284/903, 382/806 | 311/388 (80.2%) | 319/345, 3 out |
| *MLB (Retrosheet 2024)* | *91.3%, 5.6% of sends* | *96.2%, 0.6%* | *92.9%, 2.2%* | *92.1%, 3.9%* | *52.2% / 62.6% / 55.9%* | *two out: 42.5% (outfield, second open)* | | *58.9%* |

† The first scope: every runner deciding on home with two out, including the runner from first on a double. It does not differ from the built scope for a runner from second.

- **The corners: 83.0% → 93.1% sent, against MLB's 92.1%, and 1.4% → 4.3% of sends put out, against 3.9%.** With about 175 chances, the standard error is about 2 points on the share sent. Split by field, the outs fall the other way round from MLB's (left 0, right 7, against 28 and 11 in MLB). On 7 outs that is not distinguishable from noise.
- **Center: 98.3% → 99.2%** (one more runner). MLB's 96.2% is under both. The sim's center was already above it after the late-catch merge (the runner-start note measured 96.5% on its older build).
- **0–1 out, by field, and first to third, by outs:** the same within a runner or two at every value. The value only applies with two out to a runner from second going home. Center with 0–1 out is 72%, against MLB's 62.6%: unchanged here, and a separate gap, which grew with the late-catch merge.
- **The outs among the new sends.** Of the 7 at right, 3 are close races lost by 0.05–0.12 s against the actual throw (for example seed 9 play 181, seed 1182 play 143). The other 4 are the runner-start note's "trapped" runners. Seed 1268 play 139 is new at −0.2: he goes at third at 2.77 s, sees the throw at about 3.3 s 5 m past the bag, and turns back. He brakes from 8.1 m/s through 1.7 s and 6.6 m. He is tagged 0.9 m short of third at 7.42 s. Had he gone on, he would have been out at home by about 0.2 s. A real runner would have been out at home or in a rundown, and the league counts that as a sent runner put out (`2XH`). That he is tagged braking onto the bag is Q39 (the dive back).

## 3. The tests

- `bb_agents::running::advance_tests::the_two_out_send_from_second_takes_the_close_race`: a race 0.1 s behind the ball alone, which is 0.15 s in hand with the tag. Against the note's break-evens (41.5% with two out, 73.3% with one), the general caution holds him; the note's cut-off is 0.16 s. The two-out send takes him, and with one out he holds. At 0.7 s behind he holds. First to third, first to home, the batter-runner and second to third keep the general caution.
- `live::tests::with_two_out_the_runner_from_second_is_sent_unless_clearly_beaten`: the runner from second rounds third at 8.1 m/s and the left fielder has just secured the ball on the line from left field home. The race is set by a benchmark independent of the runner's read: his legs against the fielder's grade-50 exchange and a clean `solve_throw` at his arm. Over 200 reads (plays), at 0.1 s behind with two out he is sent on 187, against 111 at the old caution; with one out on 41. At 0.7 s behind he is sent on 2 (14 at 0.6 s, where his read's noise still sends a few).

## 4. The standard run

On `18226df` after a touch, with `sim-cli` rebuilt, `cargo test --workspace` gave 549 passed and 0 failed. The before figures are the late-catch review-fix row of the [ledger](../metrics-ledger.md); no sim source has changed since `a9508e3`. Seeds 0 [1000]:

| | Before | After |
|---|---|---|
| Runs/team-game | 4.12 [4.43] | 4.13 [4.45] |
| GB/LD/FB/PU hit | .307/.671/.099/.006 [.312/.664/.100/.008] | .307/.671/.099/.006 [.312/.665/.100/.008] |
| HR/PA, K%, BB% | 2.73%, 22.6, 8.1 [2.86%, 22.0, 8.5] | 2.73%, 22.6, 8.1 [2.88%, 22.0, 8.5] |
| Reached on BIP | 33.7% [33.8%] | 33.7% [33.8%] |
| PA/team-game, pitches/game | 37.7, 237 [38.2, 240] | 37.6, 237 [38.3, 240] |

All of these are within noise. Every digest changed, because a different send changes the rest of the game. About 12 more runners from second score with two out in 600 games, and about 5 more are put out. That is about +0.01 runs a team-game, the size of the change measured.

## 5. Open

- First to home on a double with two out: 92.3% against Retrosheet's 58.9%, and 61–67% with 0–1 out against 31–35%. The sim's opportunity count is also skewed toward two out (350 of 549, against MLB's 885 of 2,270). This was not investigated here; it may be the scorer's ruling or the race, and it belongs in TASKS §1.
- Center with 0–1 out: 72% against 62.6% (unchanged here).
- Per-coach and per-team aggressiveness, which would shift `send_two_out_z` with `caution_z`, is not built.
