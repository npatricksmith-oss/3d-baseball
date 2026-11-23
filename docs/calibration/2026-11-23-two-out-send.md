# The two-out send from second (Q38 (a)): fit and effect (2026-11-23)

Branch `work/two-out-caution`, from main `2a9e580`. The owner chose Q38 (a) on 2026-11-23: **the third-base coach's two-out pre-decision, "send unless the runner believes he is clearly beaten", with the caution set by outs and fitted to the league's two-out sends.** The diagnosis is in the [runner-start note §1](2026-11-23-runner-start-open.md#1-two-out-sends-from-second-on-singles). The rule is in agent-models §6.3, "The two-out send from second".

## 1. What was built

- `AdvanceParams::send_two_out_z` [E] = **−0.2** (`bb_agents::running`). With two out on the board as he sees them, a runner who started the play on second and decides on home uses it in place of `caution_z` (1.0). This covers his decision at third (`runner_decide`) and, on the way home only, his relook between third and home (`runner_relook`): there his chance of getting back to third keeps `caution_z` (`AdvanceParams::chooses_between`; review P3-1). `AdvanceParams::for_decision(start, next, outs)` picks the caution. The agents see no world state, and no new draw or RNG purpose is added.
- `Game::with_advance` (calibration only, like `with_runner_start`), and `BB_SEND_TWO_OUT_Z=<z>` in the `runner_timing` and `runner_rates` examples for the sweep. `runner_timing` now also counts runners put out on singles and the runner from first on doubles.

**Band and mechanism (calibration-targets §2.3, item 3; §2.4).** The band: Retrosheet 2024, two-out runners from second on outfield singles, by fielder: **91.3% sent from left, 96.2% from center, 92.9% from right; 5.6%, 0.6% and 2.2% of the sends put out at home** (runner-start note §1, recounted here with `retro_send2.py`). The corners pooled: 1,002 sent of 1,088 (92.1%), and 39 put out (3.9% of those sent). **This is an outcome fit** (review 2026-11-23, P2-1): one parameter, matched to the share of the sends and the share of them put out, and not a repaired mechanism. The first version of this note said that the held corner runners were 0–0.27 s behind the ball alone, so that the catcher's 0.25 s tag would have made most of them safe. That was measured against `runner_timing`'s throw benchmark, which is about 0.28 s slower than the throws the sim actually makes (§2a). Against the real throws most of those runners would have been out at home, so the holds were mostly right by the sim's own race. Why the sim's corners send less than the league's at the general caution is therefore not shown. The coaching rule (the coach sends with two out unless the runner is clearly beaten) is the reason for the parameter. The league's rates set its value. Only one parameter changed, so there is no compensating pair. But the runner's read at third is itself about 0.2 s hopeful (§2a): if that read is corrected, this value has to be fitted again, and it will come out lower.

**Scope: only the runner from second going home.**
- **The source.** The coaching rule is about this send: "with two out the coach decides before the pitch to send the runner from second home on a single" (Guilliams, decision-model note §1.6).
- **First to third with two out** is not touched. It already runs at 47.7% of all singles with a runner on first. Retrosheet 2024 has 42.5% (outfield singles with second open; `retro_other.py`; the sim's count also includes infield singles and second occupied).
- **First to home on a double with two out** is not touched either. It is already 92.3% (323/350), against Retrosheet's 58.9% (521/885). That gap is separate and is not caused by this caution. A first version that applied the two-out value to every runner deciding on home moved it by at most +1.7 points (to 94.0% at 0.1), within noise.
- **The batter-runner** never decides on home on a single.
- **With 0–1 out** the general caution stands.

**The value, and why it is below zero.** At 0.1 and 0.3 the corners stopped at 89–90%. At −0.2 they reach 93.1%. **The sweep supports anything from about 0.1 to −0.3** (89.5%, 93.1% and 93.7% sent, 2.6%, 4.3% and 4.9% of sends out, against 92.1% ± 2.0 and 3.9%; review P3-3); −0.2 is a pick inside that range. At −0.2 he goes when Φ(margin/σ + 0.2) beats the 41.5% break-even. That is, he goes when he believes he is no more than about 0.08 s behind ball plus tag. Before, he went only when he believed he was 0.16 s ahead. This is the coach's "send unless clearly beaten".

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

- **The corners: 83.0% → 93.1% sent, against MLB's 92.1%, and 1.4% → 4.3% of sends put out, against 3.9%.** With about 175 chances, the standard error is about 2 points on the share sent.
- **Split by field, the change is not even** (review P2-1, recounted from these CSVs). All 7 corner outs are at right and none at left. If the two fields shared one out rate, a split that lopsided would come about 2% of the time (two-sided; 86 of the 163 sends at right; hypergeometric, p = 0.010 one-sided). MLB has it the other way round (left 5.6%, right 2.2%). Below 0.3, every runner the lower caution adds at right ends as an out or a turn-back, and none scores. The run gain comes from left:

  | z | Right: sent, scored, out | Left: sent, scored, out |
  |---|---|---|
  | 1.0 | 74, 71, 2 | 72, 70, 0 |
  | 0.3 | 82, 77, 4 | 72, 70, 0 |
  | 0.1 | 82, 77, 4 | 71, 69, 0 |
  | −0.2 | 86, 77, 7 | 77, 74, 0 |
  | −0.6 | 88, 77, 9 | 81, 75, 2 |

  Why right differs is not traced. The benchmark's gap to the real throws is about the same at left and right (median 0.26 and 0.29 s, review P2-1).
- **Center: 98.3% → 99.2%** (one more runner). MLB's 96.2% is under both. The sim's center was already above it after the late-catch merge (the runner-start note measured 96.5% on its older build).
- **0–1 out, by field, and first to third, by outs:** the same within a runner or two at every value. The value only applies with two out to a runner from second going home. Center with 0–1 out is 72%, against MLB's 62.6%: unchanged here, and a separate gap, which grew with the late-catch merge.
- **The outs among the new sends.** Of the 7 at right, 3 are close races lost by 0.05–0.12 s, after the tag, against the actual throw (for example seed 9 play 181, seed 1182 play 143). The other 4 are the runner-start note's "trapped" runners. Seed 1268 play 139 is new at −0.2: he goes at third at 2.77 s, sees the throw at about 3.3 s 5 m past the bag, and turns back. He brakes from 8.1 m/s through 1.7 s and 6.6 m. He is tagged 0.9 m short of third at 7.42 s. Had he gone on, he would have been out at home by about 0.2 s. A real runner would have been out at home or in a rundown, and the league counts that as a sent runner put out (`2XH`). That he is tagged braking onto the bag is Q39 (the dive back).

## 2a. The runner's read of the throw home (review P2-1)

The probe: temporary trace hooks in `runner_decide` and `runner_relook` (`probe-hooks-q38.diff`; they only print) on the same 600 games at −0.2, and `scripts/an_read.py`. For every two-out corner chance whose first look at third saw the ball held, they record his believed ball home and its parts (his read of the release and of the flight). His look at the throw after release is recorded too. `scripts/zz_q38_release.rs` gave where the fielder released and where the catcher took the ball. 54 corner chances had a held-ball look at third (42 sent, 12 held, 6 of the 7 outs). On 10 of them the actual throw went straight home, so they can be compared:

| Median (mean), n 10 direct throws home | |
|---|---|
| His believed ball home at third − the actual ball home | **+0.21 s (+0.19)** |
| … of which his read of the release − the actual release | −0.01 s (−0.01) |
| … of which his read of the flight − the actual flight | **+0.22 s (+0.20)** |
| `runner_timing`'s benchmark − the actual ball home | +0.28 s (+0.30) |
| His believed ball home at the throw's release (the relook on the thrown ball) − actual | −0.05 s (0.00) |

**His read at third of a charging outfielder's throw home is slow by about 0.2 s**, all of it in the flight. At the release he reads it right, which is why the sent runners at right turn back 0.6–0.7 s after going (§2). On the five right-field outs traced (seeds 9/181, 111/271, 1026/13, 1182/143, 1268/139):
- **About 0.1 s is distance.** He times the throw from where the right fielder stands at his look, 76.4–77.1 m from home. The fielder then carries his charge through the exchange and releases 1.9–3.3 m nearer. The catcher takes the ball 0.4–1.8 m in front of the plate.
- **The rest is the throw.** A clean throw at the fielder's true arm plus his 2.1 m/s gain, from the look's position, takes 2.34–2.37 s by the runner's own flight table. The actual flights were 2.10–2.28 s. His read uses the cut-throw table for a cutoff man in the line, which adds 0.02 s, and his arm read (σ 1.1 m/s; on these five, 2.3 m/s low to 0.1 m/s high).
- **The benchmark** has the same distance problem (the pickup point) and no momentum gain, hence 0.28 s.

The actual flights look right against the source: about 74 m in 2.1–2.3 s, against Lang 2014's 280 ft (85 m) in 2.47–2.57 s, which is about 2.14–2.23 s for 74 m at the same average speed. It is the runner-start note's benchmark flight of 2.60 s that was slow, not the sim's throws (a line is added there).

What it means for this fit:
- The runner is hopeful at third, not timid, and the real throws are quicker than the benchmark said.
- With a correct read he would hold more often, and the value fitted to the league's sends would be lower than −0.2.
- The two errors point the same way, so they are not a compensating pair. But they are not independent of the fit either.
- This job does not change the race model (owner's instruction for the fix). The read and the benchmark gap are open in TASKS §1.

## 3. The tests

- `bb_agents::running::advance_tests::the_two_out_send_from_second_takes_the_close_race`: a race 0.15 s behind the ball alone, which is 0.10 s in hand with the tag. The general caution makes that a 31% chance, clearly under the two-out break-even of 41.5% (review P3-4; the first version used 0.1 s behind, 40.1% against 41.5%). The two-out send makes it 76% and takes him. With one out (break-even 73.3%) he holds. At 0.7 s behind he holds. A loop documents the scope (it restates `for_decision`): first to third, first to home, the batter-runner and second to third keep the general caution.
- `bb_agents::running::advance_tests::the_two_out_send_does_not_make_the_way_back_look_safer` (review P3-1): between third and home with two out, worked by hand from the run table. Home believed 0.2 s short and third 0.1 s in hand: with the send's caution on the way home only he goes on (0.26 against 0.16), and with it on both ways he would turn back (0.26 against 0.39).
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

**After the review fixes** (`4514ac7`: P3-1, the way back keeps the general caution, and the tests of P3-4), after a touch: `cargo test --workspace` 550 passed, 0 failed. The standard-run reports for both seed sets are identical to the ones above in every figure but the time. The `runner_timing` CSV over the 600 games is byte-identical to the −0.2 run of §2. P3-1 changes no decision in these games: it matters only between third and home when one base is better than even and the two are close in value.

## 5. Open

- First to home on a double with two out: 92.3% against Retrosheet's 58.9%, and 61–67% with 0–1 out against 31–35%. The sim's opportunity count is also skewed toward two out (350 of 549, against MLB's 885 of 2,270). This was not investigated here; it may be the scorer's ruling or the race, and it belongs in TASKS §1.
- Center with 0–1 out: 72% against 62.6% (unchanged here).
- Per-coach and per-team aggressiveness, which would shift `send_two_out_z` with `caution_z`, is not built.
- The runner's read at third of a charging outfielder's throw home is about 0.2 s slow (§2a): he times it from the fielder's position at the look and to the plate. The benchmark in `runner_timing` is 0.28 s slow. If the read is fixed, `send_two_out_z` must be fitted again.
- All 7 corner outs are at right (p ≈ 0.02), against MLB's left-heavy split. Not traced.
- The scope is interim (review P3-2): once first to home on a double is right, try the value for every runner rounding third with two out.
