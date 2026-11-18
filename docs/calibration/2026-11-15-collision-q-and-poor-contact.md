# The sourced collision efficiency, and the poor contact that comes with it (2026-11-15)

> Superseded figures: current numbers are in the [metrics ledger](../metrics-ledger.md) (§1).

Branch `work/collision-q`, cut from main `4fe04d7`. The owner's ruling Q6 (2026-11-15, option c): the sourced sweet-spot q ≈ 0.215 is adopted on this branch together with the mechanism that gives real hitters more poor contact than ours; the branch merges only when home runs and runs are both in range. Background: [sources](2026-11-15-collision-efficiency-sources.md), [counterfactual](2026-11-15-q-counterfactual.md), [power-mix](2026-11-15-power-mix.md), [ev-la-bins](2026-11-15-ev-la-bins.md). Tools: example `sim/crates/bb_game/examples/poor_contact.rs` (one row per contact, fair or foul: station, off-centre offset, depth, lateral intercept, timing, effort) and `analysis/statcast/poor_contact.py` (every table below).

## Summary for the lead

1. **Three changes, each a physical quantity:** (a) `e_ref` 0.442 → **0.460** (q 0.215, sourced; commit `c6acc53`); (b) the barrel's inboard falloff `f_taper` 0.55 → **0.27**, from Nathan 2000's flexible-bat result (sourced, derived); (c) **partial reach** (agent-models §3.6 step 7): the batter moves his hands only g = **0.6** of the way from his middle-middle hand radius toward the one a pitch's location needs, so inside pitches are met inboard (jammed) and outside ones toward the tip. g is an honest [E], *Proposed*. (b) and (c) are commit `e66f55d`.
2. **Why (c):** Statcast's contact quality depends strongly on lateral location (squared-up ratio 0.847 over the middle 8 in, 0.699 10+ in inside, 0.770 10+ in outside), and real swing length barely changes across the plate. The sim met every location at the sweet spot by moving the hands (ratio 0.82–0.87 everywhere). Contact depth (timing) is *not* where real poor contact comes from: real ratio is flat from 12 in deep to 6 in out front.
3. **Result (fair balls, pooled seeds; real competitive swings in brackets where they differ):** under 80 mph 23.4% (real 25.0, competitive 23.1; main 19.6, q alone 17.5); mean exit speed 89.1 (88.6 / 89.5; main 89.6, q alone 91.5); hard-hit 40.1% (40.3 / 41.7; q alone 47.5); squared-up ≥ 0.8 66% (63; q alone 73), ≥ 0.9 33% (31; q alone 41), p90 0.949 (0.955). The contact-quality distribution is now close to real at every level.
4. **What gave back:** the top. 105+ 8.3% (real 9.9, competitive 10.3; q alone 10.6), barrels 6.6% (8.0), HR per fair ball 3.96% (4.54), **HR/PA 2.51% [2.79%]** (real 3.0; main 1.78). Partial reach costs some best contact on inside and outside pitches.
5. **Run environment:** **runs 4.62 [4.92]** (real 4.39; main 4.31 [4.68]; q alone 5.32 [5.57]); K% 22.4 [22.2] (22.6); BB% 8.2 [8.2] (8.2). Ground-ball hits .330 [.328] against a real .245 are the fielding side's (brief §6); at the real GB rate runs would be well under 4.39 here, so the 0.2–0.5 run excess is not the batter's to chase.
6. **Not closed:** HR/PA 0.2–0.5 points short; the shape by location (the sim's outside pitches lose too much, its slightly-inside ones too little, §4); O-Contact 39.7% moved further below its band (45–68). One test, `play_area`, failed on a play the q change created (§7), a fielding matter.

## 1. The changes and their sources

**(a) Collision efficiency** (physics-spec §8.2, A8). e_ref = q(1 + r) + r with r = m/M_eff = 0.2015 at u_ss, q = 0.215 (range 0.21–0.23) → 0.460. Sources: lab q 0.214 and 0.227 for two 34-in wood bats at 29 in (Nathan, Smith, Faber & Russell 2010, arXiv:1009.2549, Table I), 0.228 for the best 1999 MLB wood bat (BESR white paper), the real Statcast p90 squared-up ratio (sources note §3). A8 now tests the *default* parameters (it used to derive its own e and check that) and adds a lab cross-check: e_n at 29 in for a 110 mph ball on a bat at rest is 0.478 against the measured 0.489 (SD 0.009, 54 impacts), within the test's ±0.02; e_ref 0.442 gave 0.460 and fails. The oracle's A8 case is regenerated from the spec (109.4 mph). The speed slope `s_e` and `v_ref` were not changed: the lab check passes with them.

**(b) The barrel's inboard falloff** (physics-spec §8.2 f(u)). Nathan, "Dynamics of the baseball-bat collision", Am. J. Phys. 68, 979 (2000), §III C and Fig. 10 (text read from the PDF): an 84 cm wood bat swung about a point 9 cm from the knob, a 90 mph pitch, 160 mph impact speed at 71 cm. The exit speed peaks at 71–72 cm and equals the rigid-bat value there; it is close to rigid only over 68–74 cm, and "at 60 cm … v_f has dropped by about 25% relative to the rigid value", 11.5 cm (4.5 in) inboard of the peak. The sim's bat swung the same way (`f_profile_matches_nathan_2000`) gave 0.84 of rigid there with f_taper 0.55; **0.27 gives 0.75**, and ±1.2 in of the peak stays at 0.98 (Nathan's 68–74 cm band). The tip side is unchanged: Nathan gives no number beyond "drops off rapidly" past 80 cm. This was the brief's candidate 5: the sources note's "widths match" held for the plateau only; the inboard tail was too forgiving, so in the sim a jammed ball was hardly punished (the stationary-bat q peak sat 1.6 in inboard of the sweet spot at q 0.234; now 1.1 in at 0.228, with 0.215 at the sweet spot).

**(c) Partial reach** (agent-models §3.6 step 7). ρ = ρ_aim − (1 − g)(ρ_aim − ρ_ref), ρ_ref the radius a middle-middle aim needs; C keeps its lateral position, so the shortfall moves the contact along the bat. g = 1 reproduces the old games bit for bit (checked: 60-game digest fold equal). The batter's decision model does not know the shortfall (jamming is not foreseen). Spec, oracle (`swing_oracle.py`, fixture `swing.json` regenerated from it; new oracle test), Rust test `partial_reach_jams_inside_and_reaches_outside`.

- *Why this mechanism* (Statcast 2025, twelve dates, competitive swings ≥ 60 mph, 368 batters with ≥ 20 swings): by pitch location across the plate, fair-ball ratio (≥ 0.8 share) 0.770 (43%) 10+ in outside, 0.823 (63%) 4–10 in outside, **0.847 (72%) middle**, 0.801 (56%) 4–10 in inside, **0.699 (27%) 10+ in inside**. By the lateral intercept relative to the batter's own mean (correlated −0.96 with pitch location) the same shape (table §3). Real swing length by location: 7.65 / 7.36 / 7.25 / 7.17 / 7.25 ft outside → inside, nearly flat; the planned v2 swing lengthened 0.46 ft from the inside to the outside third (oracle), so its hands do the reaching.
- *Why g = 0.6:* nothing measures how far a hitter moves his hands for location. Swept in games at q 0.215 and f_taper 0.27 (300 + 300 games each):

| g | < 80 | EV mean | 95+ | 105+ | ratio ≥ 0.8 | ≥ 0.9 |
|---|---|---|---|---|---|---|
| real, competitive swings | 23.1 | 89.5 | 41.7 | 10.3 | 63 | 31 |
| 1.0 | 18.5 | 91.0 | 45.4 | 9.6 | 72 | 39 |
| 0.75 | 20.1 | 90.4 | 43.8 | 9.2 | 70 | 37 |
| 0.65 | 21.9 | 89.7 | 41.3 | 8.8 | 67 | 35 |
| **0.6** | **23.4** | **89.1** | **40.1** | **8.3** | **66** | **33** |
| 0.5 | 25.9 | 88.1 | 37.7 | 7.7 | 63 | 31 |

  0.6 is the value whose low end (< 80 mph) matches real competitive swings; 0.5 matches the squared-up shares exactly but overshoots the weak end. Close enough either way; 0.6 is Proposed. (Without the f_taper change, g = 0.5 gave nearly the same aggregate, 23.4 / 89.2 / 40.7 / 8.8 / 66 / 34: the two changes trade against each other, and f_taper is the sourced one.)

## 2. The headline table (fair balls, seeds 0 and 1000 pooled)

Real: Savant 2025 twelve dates, fair balls with a tracked bat speed (8,758; the ratio columns over bat ≥ 60 mph). Real competitive-only (q-counterfactual §3): < 80 23.1, mean 89.5, 95+ 41.7, 105+ 10.3.

| group | n | <80 | 80-85 | 85-90 | 90-95 | 95-100 | 100-105 | 105-110 | 110+ | EV mean | p90 | 95+ | 105+ | barrel | LA of 100+ | ratio | SD | ≥0.8 | ≥0.9 | p90 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| real | 8758 | 25.0 | 9.4 | 11.1 | 14.2 | 15.7 | 14.7 | 8.1 | 1.8 | 88.6 | 104.9 | 40.3 | 9.9 | 8.0 | 12.3° | 0.821 | 0.122 | 63 | 31 | 0.955 |
| main | 31857 | 19.6 | 9.0 | 13.4 | 18.9 | 20.3 | 13.3 | 4.7 | 0.8 | 89.6 | 102.9 | 39.1 | 5.5 | 5.0 | 9.9° | 0.829 | 0.106 | 70 | 31 | 0.934 |
| q 0.215 alone | 32218 | 17.5 | 7.6 | 11.3 | 16.1 | 19.8 | 17.1 | 8.3 | 2.3 | 91.5 | 105.2 | 47.5 | 10.6 | 7.8 | 10.1° | 0.847 | 0.109 | 73 | 41 | 0.954 |
| **branch** | 31171 | 23.4 | 9.1 | 11.9 | 15.5 | 17.5 | 14.3 | 6.6 | 1.7 | 89.1 | 104.2 | 40.1 | 8.3 | 6.6 | 10.8° | 0.825 | 0.118 | 66 | 33 | 0.949 |

HR per fair ball: real 4.54%, main 2.61%, q alone 4.50%, branch 3.96%.

**Toward or away** (branch against main): < 80 toward (19.6 → 23.4; competitive 23.1 reached); 80–100 rows toward (90–100 was 39.2, now 33.0 against 29.9); EV mean toward (89.6 → 89.1, real 88.6 / 89.5); p90 toward (102.9 → 104.2, real 104.9); 95+ toward (39.1 → 40.1); 105+ toward but short (5.5 → 8.3, real 9.9); 110+ on it (1.7, real 1.8); barrels toward, short (5.0 → 6.6, real 8.0); LA of 100+ toward (9.9 → 10.8°, real 12.3); ratio mean, SD, ≥ 0.8 toward (SD 0.106 → 0.118 against 0.122; ≥ 0.8 70 → 66 against 63); ≥ 0.9 slightly away (31 → 33, real 31); ratio p90 toward (0.934 → 0.949, real 0.955); HR per fair ball toward, short.

**Exit speed of balls squared up ≥ 0.9, by bat speed** (n: mean, p90):

| bat mph | real | main | q alone | branch |
|---|---|---|---|---|
| 60–66 | 353: 94.6, 99.5 | 92.3, 95.3 | 93.5, 97.2 | 1015: 93.2, 96.8 |
| 66–70 | 644: 99.2, 103.7 | 96.6, 99.5 | 97.9, 101.4 | 3010: 97.8, 101.2 |
| 70–74 | 903: 102.8, 107.2 | 100.7, 103.6 | 102.0, 105.6 | 3657: 101.9, 105.4 |
| 74–78 | 588: 106.3, 110.1 | 104.9, 107.9 | 106.2, 109.7 | 2048: 106.1, 109.6 |
| 78+ | 135: 110.4, 114.6 | 109.6, 113.0 | 111.0, 114.8 | 609: 111.1, 115.3 |

Toward real at every band (the q change); still 1–1.5 mph low under 74 mph of bat speed.

## 3. Contact quality by where the ball is met

Lateral intercept relative to the batter's mean (in; − = closer to the batter, i.e. inside), fair balls: ratio mean (≥ 0.8 share).

| bin | real | main | q alone | branch |
|---|---|---|---|---|
| < −6 | **0.764 (44%)** | 0.852 (79%) | 0.871 (82%) | 0.786 (55%) |
| −6 to −3 | 0.834 (68%) | 0.838 (73%) | 0.855 (77%) | 0.865 (81%) |
| −3 to −1 | 0.846 (72%) | 0.827 (70%) | 0.846 (73%) | 0.866 (81%) |
| −1 to +1 | 0.849 (72%) | 0.829 (70%) | 0.846 (73%) | 0.860 (78%) |
| +1 to +3 | 0.849 (72%) | 0.828 (69%) | 0.845 (73%) | 0.840 (72%) |
| +3 to +6 | 0.835 (68%) | 0.827 (68%) | 0.843 (71%) | 0.809 (61%) |
| > +6 | **0.795 (52%)** | 0.802 (59%) | 0.821 (64%) | 0.746 (34%) |

Contact depth relative to the mean (in, + out front): ratio mean.

| bin | real | main | branch |
|---|---|---|---|
| < −12 | 0.834 | 0.888 | 0.865 |
| −12 to −6 | 0.824 | 0.875 | 0.846 |
| −6 to −2 | 0.823 | 0.866 | 0.843 |
| ±2 | 0.835 | 0.852 | 0.840 |
| +2 to +6 | 0.838 | 0.829 | 0.832 |
| +6 to +12 | 0.808 | 0.773 | 0.796 |
| > +12 | 0.710 | 0.647 | 0.670 |

- **Depth (the owner's timing idea, candidate 2).** Real contact quality does not fall with lateness: the ratio is flat from 12+ in deep to 6 in out front, and only far out front (+12) collapses. Real bat speed does fall when deep (68.0 mph at −12 or deeper against 72.6 at the mean and 73.4 at +6–12; main's sim 70.2 / 71.4 / 71.2), which is a bat-speed effect, not a contact-quality one. So the swing's timing bump (w_b, τ_b) is not the source of the missing poor contact; at most it is 2–4 mph too gentle in bat speed for deep contact, which would lower exit speed for jammed-and-late balls. Not changed (it would also change swing length, which is fitted); the branch's depth table is now close to real in shape (deep contact was too good on main: 0.888 against 0.834).
- **Lateral (candidate 1).** Fixed in aggregate by partial reach; the shape is not right: the sim's poor contact is now too much outside (> +3) and too little at −6 to +1. §4.
- **Along the barrel (candidate 1, sourced).** Higuchi et al. 2016 (PLoS ONE 11(2): e0148498, CC BY; college batters, a 90.7 mph machine, outer-half pitches only, high-speed video): within-batter SD of the impact point along the bat 53.3 mm (2.1 in), off-centre (vertical) 19.2 mm; off a tee 13 and 11 mm. The sim's along-bat SD of fair contact was 2.70 in (68.7 mm) on main and is 3.08 in (78.3 mm) now; its off-centre SD 23.4 / 23.2 mm. Both are already wider than the machine's (games add pitch types, locations and misreads), so neither a wider station spread nor a wider vertical miss (candidate 3) is supported by this source; the spread was not the problem, the barrel's response to it and the location dependence were.
- Station (fair balls): mean +1.17 in toward the tip (main +1.29), SD 3.08 (2.70); 9.8% more than 3 in inboard (5.5%); 10.8% at the tip end (8.8%). Of balls under ratio 0.8, 26% are inboard of the sweet spot (14% on main): jammed contact now exists.

## 4. What is still off, and why

- **HR/PA 2.51% [2.79%] against 3.0%; 105+ 8.3% against 9.9–10.3; barrels 6.6 against 8.0.** The reach shortfall costs best contact on off-centre pitches too. Candidates, none changed: the launch angle of hard-hit balls (10.8° against 12.3°; ev-la-bins found the 100+ balls flatter, worth about a third of the HR gap there); the spray direction gap (PLAN D20; ev-la-bins §4, about a quarter of that gap); bat speed at contact (+0.8 mph mean in real, power-mix §4).
- **The shape by location.** The sim's contact sits on average 1.2 in toward the tip (its batters are early, −20 ms on outside pitches in `foul_diag`, a perception matter, swing note §3), so a shift inboard first *improves* contact (slightly inside pitches 0.865 against real 0.834) and a shift outward goes off the end too soon (> +6 in: 0.746 against 0.795). Separate gains, 0.45 inside and 0.85 outside (tested at q 0.215 without the f_taper change), matched both edges (0.770 / 0.801) but left the middle too good (≥ 0.8 71% overall): the middle is at least partly the timing bias's. *Qualified after the Codex review (P2-1):* timing is a shown contributor (in the noise-free cage a 20 ms earlier swing moves an inside contact from −3.25 in to +0.70 in of the sweet spot, ratio 0.842 → 0.960, and an outside one from +2.79 to +4.60 in, 0.880 → 0.712), but it is not shown to be the whole residual: contacts within ±5 ms of nominal timing still have a middle-plate ratio of 0.877 against the real 0.847. Height, lateral aim and selection errors are not separated. Question 2 below.
- **Swing length by location in games** barely moved: outside → inside 7.98 / 7.63 / 7.46 / 7.23 / 6.89 ft (MLB 7.45 / 7.31 / 7.23 / 7.19 / 7.13; the swing note's v2 before the refit 7.95 / … / 6.76). In games the batters' timing bias by location dominates it, so this check only confirms the direction, as planned in the oracle (0.46 → 0.28 ft between the thirds' centres).
- **Plate discipline** (`run_env_diag 300 0`, seed-0 set; main from the ledger's refit row, same games): Contact% 65.9 → 65.3 (band 70–83, away), Z-Contact 72.9 → 73.0 (79–91), **O-Contact 42.8 → 39.7** (45–68, away: outside pitches met beyond the tip now miss), SwStr 16.1 → 16.3% (8–15, away), O-Swing 24.5 → 24.1, Z-Swing 65.5 → 65.4, pitches/PA 3.11 → 3.16. K% 21.5 → 22.4 [22.2] (22.6, toward).
- **Slow swings (candidate 4) were left out.** Real tracked fair balls from swings under 60 mph are 3.7%; they account for 1.9 points of the real < 80 share (25.0 against 23.1 competitive) and 0.9–1.0 mph of its mean. The branch matches the competitive-only numbers, so adding them (a two-strike protection decision model) would take the sim to the all-swing numbers: about +2 points under 80 and −1 mph mean, the remaining difference to the full real sample.
- **Ground-ball hits** .330 [.328] against .245: unchanged by this work in kind (the q change raised them, the reach change brought them back, main .322 [.326]); fielding's.

## 5. Rejected, and why

- **Wider station or vertical spread** (candidates 1 and 3 as a noise change): the sim's spreads are already wider than the only measured ones (Higuchi 2016); widening them would be a fudge.
- **A steeper timing fall-off** (candidate 2 as a contact-quality mechanism): real contact quality does not depend on depth except far out front (§3).
- **Re-sourcing `s_e`, `v_ref`:** the lab cross-check passes with them; no source gives a game-speed slope.
- **f_tip:** Nathan gives no number; unchanged.
- **Asymmetric reach gains:** fit the edges, not the middle (§4); two parameters for one table. Offered as a question.
- **Slow swings:** real, but a decision model of their own, and not where the gap was (§4).

## 6. Owner questions

**Q-A. The reach gain g = 0.6 (Proposed).** *What happens now:* every batter moves his hands 60% of the way a pitch's location needs; the rest of the reach lands along the bat (jammed inside, off the end outside). *Problem:* no source measures it; it is set so the contact-quality distribution matches Statcast, which it does within 1–3 points everywhere except the top (105+ 8.3 against 10). *Options:* (a) keep 0.6; (b) 0.65 (a little more top: 105+ 8.8, < 80 21.9); (c) make it a batter trait later (plate coverage), with 0.6 as the league mean. *Recommendation:* (a) now, (c) with batter ratings.

**Q-B. The location shape.** *What happens now:* outside pitches are squared up too rarely (> 6 in out: 34% ≥ 0.8 against 52%) and slightly-inside ones too often. *Problem:* the cause is upstream (the batters' early timing bias on outside pitches puts contact toward the tip), so fixing it with separate inside and outside gains would hide a perception problem. *Options:* (a) leave it and take it up with the timing bias (swing note open item 3); (b) separate gains now. *Recommendation:* (a).

**Q-C. Merge gate.** Runs 4.62 [4.92] are 0.2–0.5 over 4.39 with ground-ball hits 0.085 above real, and HR/PA is 0.2–0.5 points short. *Recommendation:* the HR shortfall is in range of the next sourced items (launch angle of hard contact, the direction gap); the run excess is fielding's. Whether this is "both in range" is the owner's call.

## 7. Tests

`cargo test --workspace --release` passes on `e66f55d` (all targets; `--no-fail-fast` run). Python oracle tests pass (`test_oracle.py` 23, `test_swing_oracle.py` 5). Changed or added:
- A8 now checks the default parameters (q within 0.21–0.23 and 0.215 ± 0.002 at v_ref; EV 107.5–111) and the lab BBCOR at 29 in (`a8_lab_bbcor_at_29_in`).
- `f_profile_matches_nathan_2000` (75% ± 3 of rigid 4.5 in inboard of the peak; ≥ 97% within ±1.2 in; the old profile fails).
- `partial_reach_jams_inside_and_reaches_outside` (Rust) and `test_partial_reach_jams_inside_and_reaches_outside` (oracle); the exact-aim and reach-clamp swing tests now state g = 1.
- `swing_exit_speed`: the cage's three thirds no longer must agree within 4 mph (an assumption from before step 7); the middle must be the best third and the spread below Statcast's 13.8 mph (cage now about 91 / 97 / 94).
- **Fielding finding, not fixed (not this branch's code):** on the q-only commit `c6acc53`, `play_area` failed on seed 4 play 191: a foul fly over the first-base taper wall at about 6 m height, out of play where the first baseman's glove met it, 0.93 m beyond the wall plane, where the test allows 0.7 m (a 1.0 m reach from a body kept 0.3 m inside). The play does not occur on `e66f55d` (the games changed again), so the test passes there; the reach over a wall that allows 0.93 m is still in `live.rs`.

## 8. Reproduction

- Real data: the twelve Savant 2025 dates of [collision-targets](2026-11-14-collision-targets.md) in `sim/target/data/savant/` (not committed).
- Sim: `cargo run --release -p bb_game --example poor_contact -- <0|1000> 300 out.csv`, then `python3 -I analysis/statcast/poor_contact.py sim/target/data/savant label=a.csv,b.csv …`. The sweeps patched the defaults through temporary environment variables (`BB_REACH_GAIN`, `BB_F_TAPER`, an inside-only gain), not committed.
- Standard run: `sim-cli games --n 300 --seed 0` and `--seed 1000`; `run_env_diag 300 0`; `foul_diag 3000`.

## Resolution

Not yet reviewed (the lead runs the Codex review). Open.
