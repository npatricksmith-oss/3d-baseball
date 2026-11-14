# Hurried throws review

Reviewed `a4c8219..fa2cf03ee283bf17732e93b675711762543d3010` against `CLAUDE.md`, PLAN's invariants, and `docs/agent-models.md` §5.5 and §10. No P1 findings. Two P2 findings and two P3 specification discrepancies remain.

## Findings

### P2 — Record dive footwork per possession instead of reading the recovery flag

**Location:** `sim/crates/bb_game/src/live.rs:2710`, interacting with `live.rs:1823` and `live.rs:2030`.

`set_up_for` recomputes dive footwork from mutable `fd.dove`. That flag describes recovery and is cleared by `move_fielders` at `ready_at`, which capture sets to `up_at + transfer`, before the additional dive footwork has elapsed. Clearing it shortens the current throw's setup and removes its off-balance penalty. Conversely, captures run before `move_fielders`: a standing catch on the first tick after recovery still sees the old `dove = true`, renews `ready_at`, and inherits footwork from the previous dive.

**Verified:** scratch test `review_dive_footwork_lifetime` secured a low ball at 1.5 s while down, with get-up due at 2.5 s, zero catch velocity, and no turn. The initial setup correctly required `2.5 + 0.85 + 0.30 = 3.65` s. With no runner to hurry for and a receiver on the bag, the fielder instead released at **3.383333 s**, recorded as **`(hurry = 0, off_balance = false)`**. The recovery flag had removed the final 0.3 s. `review_recovered_dive_contaminates_next_capture` then set recovery to half a tick before a standing catch at 1.5 s: no new dive occurred, but setup was 1.15 s and off-balance instead of 0.85 s.

Record whether this particular acquisition required a dive/get-up, reset it on every acquisition, and retain that fact through this possession's setup. Determine it from the actual acquisition posture, including the recovery boundary. Add lifecycle coverage that runs capture, holder, and movement together; setup-only tests miss both failures.

### P2 — Do not require throw setup before choosing to carry the ball

**Location:** `sim/crates/bb_game/src/live.rs:2666`, before the carry branch at `live.rs:2681`.

The new return while getting set precedes “Closer to take it himself.” When the earlier `d_me < 8` race check declines to commit to running, the later carry branch now waits for transfer, turn, and footwork even though it will never throw. A comfortable margin on the hypothetical throw actively prevents hurrying through this gate. This adds a throwing-only delay to an already marginal running play.

**Verified:** paired scratch tests at HEAD and `a4c8219` placed the first baseman 5 m east of first, facing −Y, at time 1.65 s after securing it at 1.0 s. Transfer was 0.65 s, the pitcher covered first, and the runner was 1.2 s away. The early run check did not apply: the standing-start run estimate was 1.20834 s, before its extra 0.1 s allowance. At the old revision, `holder_act` immediately targeted first. At HEAD, it retained the current location because the unused throw required 0.77 s of setup; carrying must wait another **0.12 s**. The hypothetical throw has ample margin in both cases. Select the carry action before applying the setup/hurry gate to throws.

### P3 — Specify whether dive footwork replaces or adds to moving-away footwork

**Location:** `sim/crates/bb_agents/src/throwing.rs:78`; contract: `docs/agent-models.md:423`.

The spec lists **+0.2 s** for moving away and **+0.3 s** after a dive, without an exception when both apply. `footwork_s` immediately returns 0.3 s for a dive and discards the moving-away term. The additive reading of the binding spec gives 0.5 s. This discrepancy exists independently of the dive-state lifecycle bug above.

**Verified:** `review_combined_footwork_terms` passed catch velocity `(−5, 0, 0)`, target direction `(10, 0, 0)`, and `dove = true`; the result was **0.3**, not **0.5**. The new unit tests exercise the two cases separately. State the intended precedence in §5.5 or implement the additive contract, and cover their intersection with a literal expected value.

### P3 — Document the incoming-ball speed cutoff and minimum hurry gain

**Location:** `sim/crates/bb_game/src/live.rs:2034` and `live.rs:2668`; contracts: `docs/agent-models.md:422` and `agent-models.md:428`.

The spec applies the 3 m/s facing threshold to the fielder's running, then says he faces the incoming ball, otherwise the plate. The implementation also requires the incoming ball's **horizontal** speed to exceed 3 m/s. A slower incoming ball is treated like a resting ball. Separately, the code requires an arrival-time improvement strictly greater than **`HURRY_GAIN_MIN_S = 0.01` s**, whereas the spec only rules out hurrying when it gains no time. Both are extra decision thresholds that should be stated explicitly.

**Verified:** `review_capture_facing` captured a ball moving at `(2, 0, 0)` with a stationary fielder at `(0, 80)`. Recorded facing was the plate, `(0, −80)`, rather than the incoming-ball direction `(−2, 0)`. For a right-handed throw to second this gives 0.85 s setup instead of 0.73 s under the spec's incoming-ball rule. The 0.01 s cutoff is the explicit comparison at line 2668 and named constant at line 102.

## Verification

All experiments and added diagnostic tests were confined to Git archive copies under `/tmp/hurried-review-{old,head,probes}`, with separate Cargo target directories. Only this review was written in the tracked tree. Fresh links initially encountered the read-only default Zig cache; subsequent builds used `ZIG_GLOBAL_CACHE_DIR=/tmp/hurried-zig-cache` and `ZIG_LOCAL_CACHE_DIR=/tmp/hurried-zig-local`.

### Commands and measurements

- `export PATH=$HOME/.cargo/bin:$PATH; cd sim && cargo test --workspace --release`: **192 tests passed**, no failures; doc tests also passed. Includes crate-graph isolation, live-play tests, game accounting, and the new throwing tests.
- `cargo run --release -p bb_game --example throw_probe -- 300 0`: reproduced every printed aggregate in the note. Set throws **46.30/game, 96.3%** taken in the air; hurry thirds **0.29/96.6%, 0.29/96.5%, 1.96/92.5%**; on/off balance **34.05/14.78 per game, 96.3%/95.7%**. Second base had the highest mean hurry, 0.10.
- `cargo run --release -p bb_cli -- games --n 300 --seed 0`: reproduced **5.54 runs/team-game, 32.5% reached on BIP, .352 GB hits**, with no rules warnings.
- Ran the compiled `throw_probe` with `100 0 20 20 20`, `200 0 50 50 50`, and `100 0 80 80 80`. Reproduced the ratings table: outs/BIP **.548/.585/.606**, and highest-hurry-bin catches **87.1%/92.3%/94.7%**. That bin is hurry greater than two thirds, rather than exclusively hurry exactly 1.
- Ran `cargo run --offline --release -p bb_cli --no-default-features -- games --n 300 --seed 0` in the old copy and each scratch ablation. All rows reproduced:

| Build | Runs/team-game | Reached on BIP | GB hits |
| --- | ---: | ---: | ---: |
| `a4c8219` | 5.35 | 32.0% | .346 |
| HEAD, no footwork | 5.17 | 31.0% | .337 |
| HEAD, no turn | 5.59 | 32.6% | .349 |
| HEAD, no hurry | 5.57 | 32.6% | .354 |
| HEAD, all three off | 5.35 | 32.0% | .346 |

For the all-off build, set `TURN_S`, `AWAY_FOOTWORK_S`, and `DIVE_FOOTWORK_S` to zero and `SAFE_MARGIN_S` to `−1e9`. Ran `cargo run --offline --release -p bb_game --example digest` at both revisions: **all four digests matched**, for seeds 1, 2, 3, and 42. For example, seed 1 was `8585b5bb654cb954ecab4854f575a609c52f3cd0e357a17c38defbdc22c5695b` at both. Repeated the unmodified HEAD digest example twice; all four pairs matched there too. These sampled digests and the 300-game aggregates support the note's preservation claim; they are not an exhaustive proof over seeds.

Ran nine additional scratch tests using `cargo test --offline --release -p bb_game --lib review_ -- --nocapture`, plus the paired old-revision carry test. They verified the findings, facing capture state, motor draw equivalence, receiver waiting, and a complete two-leg relay after a real capture. The receiver-wait case released unhurried at 2.75 s. The relay secured at 5.0/8.316667/11.2 s and released at 5.866667/9.166667 s; both releases respected their earliest times. No hang or premature release was observed in these cases. A bounded search over direct/relay timing alternatives found no case meeting its unnecessary-hurry criterion; it does not establish that all route choices are optimal.

### Invariants, geometry, and test independence

The changed holder decision uses current visible positions and runner motion, its own setup/arm model, and the reference-atmosphere throw-time belief. It does not inspect a realized future trajectory or an `Executed*` value. `execute_throw` is called only after choosing the action; its output feeds launch physics. No outcome roll, new RNG purpose, unordered simulation iteration, or forbidden transcendental call was introduced. The relay estimate retains its pre-existing access to teammate arm/transfer attributes; this change does not introduce that simplification.

The scratch motor check compared the old inline expressions with `execute_throw` for **1,000 keys**, including exact speed/yaw bits and the next RNG word. All matched: speed is drawn first, then direction, with truncation still 4σ.

Independently computed the turn with Python `math.atan2` headings: rotate the initial facing by +90° for a right-hander's glove shoulder, −90° for a left-hander's, then take the shortest absolute body rotation to the target. Used the actual diamond coordinates, `27.432/√2`, without calling Rust's turn function:

| Situation, initially facing the plate | Right-handed rotation / extra time | Left-handed rotation / extra time |
| --- | ---: | ---: |
| SS `(−8, 38)` → first | 46.06494° / −0.058580 s | 133.93506° / +0.058580 s |
| 1B `(19, 22)` → second | 179.34059° / +0.119121 s | 0.65941° / −0.119121 s |

Thus the spec's handedness claims and turn signs are correct. Capture probes also confirmed plate-facing for a resting ball, movement-facing for a 5 m/s running catch, and `face = None` plus 0.7 transfer for a received throw. The slow incoming-ball exception is finding four.

The new grade-table, cardinal-turn, worked setup, and release-time assertions use useful literal expectations from the spec. The whole-game grade test supplies a behavioral direction check, although it varies all three ratings together. Some live tests are partly model-relative: `eta_for_margin` calls the production `throw_time`, and the slower/wilder test recomputes factors from the reported hurry. Those verify wiring and consistency, not an independent flight-time oracle or the hurry calculation itself. Most importantly, `shortstop_to_first` installs setup fields manually and never runs capture or `move_fielders`, explaining why the dive-state failure passes the suite.

### Parameter comparison

Apart from the findings, §5.5 and §10 agree with the implementation:

| Parameter | Verified implementation |
| --- | --- |
| ArmStrength | Table mean 37 m/s, σ 2.2 m/s per ten grades; live position means 36/39 m/s, shifted by grade as the §10 note specifies |
| ArmAccuracy | 1.2° at grade 50, ratio 1.2 per ten grades; converted to radians |
| Transfer | Table reference 0.75 s, ratio 1.08; live position references 0.65/0.85 s scaled by grade, as explicitly documented |
| Received throw | 0.7 transfer, no turn; capture velocity still contributes moving-away footwork |
| Turn | 0.12 s per quarter-turn difference from the straight-ahead case |
| Footwork | Away speed strictly greater than 2 m/s: 0.2 s; dive: 0.3 s, with precedence and lifetime issues above |
| Earliest release | 0.6 × max(transfer + turn, 0); hurry decreases linearly to zero at set time |
| Hurry speed loss | 0.10 plus 0.08 off balance, both multiplied by hurry |
| Hurry direction growth | 1.0 plus an additional 1.0 off balance, both multiplied by hurry |
| Safe margin | 0.25 s; additional 0.01 s gain floor is not specified |
| Motor noise | Speed σ 3%, then horizontal direction noise; truncation 4σ |

Speed-for-accuracy, composure, individual generated ratings, and deliberately selected one-hop throws remain documented omissions, rather than regressions in this range. The paragraph saying every throw is at full arm should now be read as every **set** throw; the new hurry equations govern early releases.

## Resolution

Resolved 2026-11-14. All four findings fixed; 195 tests pass. After the fixes runs/team-game are 5.53 (seeds 0–299; 5.54 before them).

1. **P2, dive footwork read from the recovery flag: fixed.** Each securing records `took_diving` (down as he takes it: his get-up is still ahead of him), and the set-up reads that, not `dove`. Two lifecycle tests run captures, the holder and movement together with literal expectations: `dive_footwork_lasts_until_the_throw` (released at 3.53 s, recorded off balance) and `a_recovered_fielder_owes_no_dive_footwork` (the stale-flag case: 2.23 s, on balance).
2. **P2, carrying waited for a throw's set-up: fixed.** The carry is chosen before the set-up gate and waits only for his plain transfer, as before A3 (`carrying_the_ball_to_the_bag_waits_for_no_throw_set_up`, the review's case). Whether carrying should wait for a transfer at all is older behaviour, left alone.
3. **P3, dive and moving-away footwork together: specified.** A dive replaces the moving-away term (he gets up from a standstill); §5.5 says so and the unit test has the 0.3 s case.
4. **P3, two unstated thresholds: specified.** §5.5 now gives the 3 m/s cutoff for the incoming ball and `HURRY_GAIN_MIN_S` = 0.01 s, and says "every set throw" is at full arm.

Not changed: the live tests that build their scenario with the production `throw_time` (the review calls them partly model-relative). Their assertions are literal release times from the spec; an independent flight-time oracle for throws is a separate job.
