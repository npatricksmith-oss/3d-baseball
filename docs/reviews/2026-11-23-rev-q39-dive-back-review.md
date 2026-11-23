# Review: Q39 (a), the dive back to the bag (rev-q39, 2026-11-23)

Reviewer: rev-q39 (Claude Opus, high effort), independent of the change. Range `310d6ba..0ec2704` on `work/dive-back`. Files: `bb_game` `live.rs` (`dive_tick`, `step_dive`, `touch_s`, `tag_distance`, `retouch_dive`, `runner_relook`'s dive choice, `move_runners`, `holder_act`'s tag check, five tests), `timeline.rs`, `game.rs`, `bb_ffi` (`BbDive`, `active_dives`, the timeline test), `bb_ffi.h`, the client (`BbPose` `DiveBack`, `BbDirector` routing, `NoteSlides`, `-BbDiveBackShots`); agent-models §6.4 "Diving back", data-formats §5.4, player-visuals §11, the [dive-back note](../calibration/2026-11-23-dive-back.md), the ledger row, TASKS §1.

**Verdict.** The mechanism is sound and keeps the invariants. The runner chooses the dive from his own reads, and the dive law matches its hand-worked closed form. The rules see the hand's touch, and the client puts the hand on the bag at the sim's touch time. No RNG draws were added, and a clean build reproduces the worker's standard run in every figure. The note's central claim also holds: `OutCall::margin_s` is a distance divided by top speed.

Two things are wrong, and they are linked:
- The dive has no lower bound on the margin. In 259 of 829 dives, the runner by his own read was beaten by more than a second. Most of those (175) are trapped runners committed to the way back. They sprint and dive headfirst at a fielder standing on the bag with the ball, and 162 of them are tagged in the air.
- Those in-air dives are never recorded, so the client draws them as upright runners gliding into the tag.

Neither changes a game's outcome: those runners are out either way. Both are worth fixing before merge, or carrying as the first item after it. The P3s are small.

## Findings

### P2-1. Hopeless returns dive: there is no lower bound on the margin, and the turn-cap path (`c167aee`) makes almost all of them

*Where:* `sim/crates/bb_game/src/live.rs:8151` (`r.dive_to = (r.goal == from && from >= 1 && m_back < DIVE_CLOSE_S)…`), reached for a capped, committed runner through `live.rs:8142`. `retouch_dive` at `live.rs:8663` has the same open lower end (`ball - mine < DIVE_CLOSE_S`). Spec: agent-models §6.4 "When".

*What:* The spec's reason for the dive is "whenever braking could lose him the race". The code dives for any margin under 0.5 s, however negative. A trapped runner (`!go_on && !go_back`) always has `m_back` below the go-back threshold. So every trapped runner heading back dives, including those held to the way back by `MAX_TURNS`.

I instrumented a copy of `0ec2704` (prints only; the standard-run report is byte-identical to the unmodified build) and ran the standard 600 games (`games --n 300 --seed 0` and `--seed 1000`). Every take-off was matched to the decision that chose it:

| Source of the dive (his margin by his own read) | Dives | Hand reached the bag | Put out in the air | Play ended in the air |
|---|---:|---:|---:|---:|
| Relook, not trapped, 0 to 0.5 s (the intended close race) | 49 | 49 | 0 | 0 |
| Relook, trapped, turned back by this look | 40 | 40 | 0 | 0 |
| Relook, trapped and **capped** (`MAX_TURNS`), margin < −1 s | 175 | 13 | **162** | 0 |
| … capped, −1 to −0.5 s | 25 | 6 | 19 | 0 |
| … capped, −0.5 to 0 s | 3 | 2 | 1 | 0 |
| Retouch, believed margin 0 to 0.5 s | 366 | 366 | 0 | 0 |
| Retouch, −0.5 to 0 s | 50 | 47 | 0 | 3 |
| Retouch, −1 to −0.5 s | 39 | 27 | 10 (appeals) | 2 |
| Retouch, < −1 s | 82 | 64 | 15 (appeals) | 3 |
| **All** | **829** | **614** | **207** | **8** |

- 182 of the 207 in-air outs are tags on capped runners. By their own read, 181 of them were beaten by more than 0.5 s and 162 by more than a second. Examples: seed 266, play 148, runner 0x104, margin −2.8 s. Seed 153, play 20, runner 0x103, −2.08 s. Seed 228, play 66, runner 0x109, −1.96 s. All are game seeds (`LiveInput::seed`) from the `--seed 0` set.
- The note's line "more runners left their feet and were tagged in the air than were out at the bag" understates this. It is 207 of 829 dives, one in four.

Would a real player do this? A runner who sees the fielder already on his bag with the ball does not sprint and leave his feet a body-length in front of him. He pulls up and tries to stay alive in a rundown (Q11). Braking into the tag was not right either, but it at least did not show a dive no runner would choose.

*Fix:* Dive only when the dive can change the result by his own read: for example, `−DIVE_SAVE_S < m_back < DIVE_CLOSE_S`, with the saving taken from the dive law or a named [E] of about 0.4 s. Use the same lower bound in `retouch_dive`. Otherwise brake as before (the rundown stand-in). Say so in §6.4 "When", and give a test: a capped runner beaten by 1 s brakes. This should leave the outs unchanged; check that with one standard run.

### P2-2. Dives that end in the air are not recorded, so the client draws a quarter of dives as an upright runner gliding into the tag

*Where:* `live.rs:8622`: `RunnerDive` is pushed only at the hand's touch. `timeline.rs` emits `Dive`/`BaseTouch`/`DiveUp` only from `runner_dives`. `bb_ffi` `active_dives` needs a `Dive`…`DiveUp` pair.

*What:* From take-off the sim has the runner off his feet, gliding at constant speed, and tags him on his laid-out body (`tag_distance`). If he is tagged or appealed out before his hand arrives (207 in 600 games), or the play ends in the air (8), there is no record. The timeline has no `Dive` marker, `BbFrame.dives` has nothing, and the client draws the sim's path point moving at glide speed (4–7 m/s) with a running figure, which stops dead at the out. The physics and the picture disagree: the sim's tag geometry used a body that the client never shows.

*Fix:* Record the dive at take-off. Make `t_touch` optional, and add the time it ended (the out, or the play's end). The timeline then emits `Dive` and a `DiveUp` for a dive with no touch: the runner lies where he was tagged and gets up after `DIVE_BACK_UP_S`, and the clip lasts until then, as for a fielder. The client draws the glide and the lying runner from `since_s`/`up_in_s`, with no hand on the bag. Fixing P2-1 removes most of these dives, but genuine close dives tagged short will remain.

### P3-1. A dive cancelled inside braking distance makes the runner overshoot his own bag

*Where:* `live.rs:8151`, where every look recomputes `dive_to`, together with `live.rs:8437`. Spec: agent-models §6.4 "When" says "a new look that sends him on cancels the dive; so does any other change of goal". The code also cancels it when a new look keeps him going back with a margin of 0.5 s or more.

*What:* 104 run-in dives were cancelled this way before take-off. 6 of those runners were already inside their braking distance, so the braked path (`step_to_stop`) carries them past the bag and back. Example: seed 139, play 48, runner 0x7. He was 3.54 m out at 6.93 m/s, and his braking distance was 5.06 m. His margin went from −0.29 s to +1.08 s. A real runner in that spot slides or dives into the bag anyway.

*Fix:* Keep a chosen dive while the goal is unchanged once he cannot stop short, or simply keep it whatever the later margin, since diving is never slower. Write the rule into the spec.

### P3-2. The client assumes every take-off is 1.8 m out; 17 of 829 were nearer (as near as 0.59 m)

*Where:* `client/Source/BaseballClient/BbDirector.cpp:4199` (`Left = 1.8 * …`); data-formats §5.4 line 396 ("1.8 m … short of the bag at take-off").

*What:* He leaves his feet at whatever distance he has when he first moves toward the bag within `DIVE_BACK_M`. A retouch called late, or a decision made already close, starts the glide nearer: 17 take-offs were inside 2.0 m and 7 inside 1.0 m. For those, the drawn hand starts up to 1.2 m farther out than the sim's and closes faster, though it still lands at the touch. Separately, at every take-off the drawn origin steps 0.4 m forward (from the path point to the hand). At the touch, the sim's path point steps 0.4 m onto the bag.

*Fix:* Carry the take-off distance in `BbDive`, or derive it from the `Dive` marker's `p` and the bag. Correct the data-formats sentence.

### P3-3. Docs and test text that do not match the code

- agent-models §6.4, Tests, `docs/agent-models.md:838`: "with the ball due to be tagged on him 0.13 s after his braked arrival". In `back_to_third` (`live.rs:20745`) the tag can come 0.13 s after the dive's hand-worked touch, which is about 0.4 s *before* his braked arrival. A tag after his braked arrival would leave the braked runner safe too, and the test's first assertion would fail. Also, by the new note's own correction, the "typical case" of 0.13 s was a runner 1.15 m short, not a 0.13 s race. Name it a constructed close return.
- `live.rs:20803`: the comment "a runner on his feet at that path point is 1.0 m from him as well" describes an assertion the test does not make. The test checks a point 1.0 m beyond his feet.
- `retouch_dive` (`live.rs:8672`) takes the throw from the catcher's position when the runner sees the call, not "from the catch point" as the doc comment and §6.4 say. The difference is small (he has moved a step or two), but the text should say which.
- Dive-back note §2/§summary 2: quantify the in-air outs (207 of 829 in the standard run), and say where they come from (P2-1).

### P3-4. Note §3's "not a fault" for the runner from third rests on matching totals that hide divergent rows

*Where:* dive-back note §3; agent-models §6.4 last bullet; TASKS §1.

*What:* The cohorts match well enough. The Retrosheet script takes a runner on third, 0–1 out, not forced, a ground ball first fielded by an infielder, no bunts. The probe takes the same with launch angle under 10°, the first securing fielder 1–6, and no catch. The aggregate does match (56% against 56% scored, and 11% against 12.4% put out). But by first fielder the sim is far off in both directions:

| First fielder | Sim scored | MLB scored |
|---|---:|---:|
| Pitcher | 10% | 48% |
| First | 39% | 54% |
| Short | 76% | 55% |

MLB's cohort also includes infield-in plays, which the sim does not model; they move MLB's outcomes toward outs at home. The reading that MLB's `3XH` outs include rundowns that end back near third is plausible but unchecked.

*Fix:* Keep the conclusion that the break is not a fault overall, but list the per-fielder divergence (short and pitcher) and the infield-in confound as open, not as "left as found" for the comebacker alone.

I checked §4's sim cohort against the probe's rows. All 50 runners from second put out going back were at 0–1 out with an infielder first to the ball (a forced runner cannot go back), so the comparison with Retrosheet's 0.7 per 600 games is like with like.

### P3-5. The C ABI grew with no size or version handshake

*Where:* `sim/crates/bb_ffi/include/bb_ffi.h:28`, `bb_ffi/src/lib.rs` `BbDive`; `client/Source/BaseballClient/BbSim.cpp:20-40`.

*What:* `BbDive` went from 20 to 24 bytes, so `BbFrame` grew by 16. The header and Rust agree: `agent` u16, `base` u16 (was `_pad`), `since_s`, `up_in_s`, `dir[2]`, `touch_in_s`. But the client loads `libbb_ffi.so` at run time and checks only that the symbols exist. A client binary built before this change, run against a library built after it, has `bb_timeline_sample` write 16 bytes past the caller's `BbFrame`. This is the existing design (`n_dives` was added the same way), but this change is the kind it bites, and worktrees rebuild the library and the client separately.

*Fix:* Export `bb_abi_version()`, or the struct sizes, and refuse to start on a mismatch.

### P3-6. Client cosmetics (confirmed) and one coverage gap

- **Upright at take-off** (the worker's flaw 1): confirmed in all three take-off shots (`diveback_00_1`, `01_1` for seed 22, `00_1` for seed 31). The figure is still in its running pose; the 0.12 s blend has not started.
- **The covering fielder overlaps the runner** (flaw 2): confirmed in all three dives photographed. Seed 22's second baseman stands on the runner's hand at the touch and the runner rises up through him (`00_3`–`00_5`). The first baseman at first does the same (`01_5`). The third baseman steps through the lying runner at third (seed 31, `00_3`).
- **Coverage:** while in the air and while down (1.3 s), the diving runner skips `settle_guard` and `runner_traffic`, so he can neither give way nor go on. agent-models §6.4 "Where bodies stop" says no decision path bypasses that guard. I found no way for a trailing runner to be forced onto his bag, and the standard run has `rules warnings: {}`. But the 3,000-layout property test never puts a runner in a dive. Add a diving or lying runner to its layouts.

## Answers to the job's questions

1. **Invariant 2 and the decision.** Between bases, the choice uses `m_back` from `runner_relook`. That is his keyed read of the ball's arrival (`runner_ball_races`, sub-index 2n + 1), his own braked return time (`run_time_to_stop`), the typical tag, and his timing-error draw. The retouch choice uses the believed throw: a typical transfer, plus a typical throw from the catcher he can see to his bag, plus the seen cut. Neither reads the true ball, its destination or a tagger's true timing. The turn-cap path uses the same `m_back`. The answer to whether a real player would make each dive is no for the capped runners: P2-1.
2. **Rules.** The touch is the hand crossing the base coordinate, and the touch fact (`BaseTouched`) is emitted then. A tag is no play once the hand is within 0.3 m (`ON_BAG_TAG_M`, the standing runner's tolerance). A tag before then is an out (OBR 5.09(b)(4)). The tag reaches the segment from the hand back 2.0 m. `tag_distance` is correct geometry. The body appears in full at take-off (1.6 m behind his path point), which matters only for a tagger trailing him. On an appeal, the fielder's touch with the ball is compared with the hand's touch by the order of the facts, and `holder_act` runs before `move_runners`, so the fielder wins a tie, as for a standing runner (5.09(c)(1)). A returning runner is never forced back, so no force play arises at that bag. The deferred P3 test (`a_runner_tagged_between_bases_after_crossing_a_bag_that_is_not_his_goal`) is sound: touch and departure on one tick, then the tag, then the rules' tag out.
3. **Down.** At the touch his path point is the bag (`s = bs`), and the rules have his touch, so he is protected. He makes no decision until up; the test covers this, and on being up `base_relook` picks up the looks he missed. He cannot be down off the bag. Tagged in the air, he is out and skipped thereafter (`move_runners`, `holder_act`, `sample_runners` all skip out runners). His `dive` field stays set but is never read again. A play ending in the air leaves no record: P2-2.
4. **Determinism.** No new draws and no `Purpose`. `retouch_dive` uses `m::hypot`, `dive_tick` uses arithmetic only, and there is no `HashMap`. Only the test helper uses `f64::exp`. The full suite passes after `e4b1a40` (see Verification). The standard run is identical to the worker's.
5. **Timeline and FFI.** The header and Rust agree (P3-5 for the missing handshake). Old timelines carry no `BaseTouch` (first emitted on this branch), and fielder `Dive` markers have `base` 0, so they decode to `touch_in_s` 0. The saved-play check passes, and two old recorded saved plays open.
6. **Client.** The hand is on the bag at the sim's touch: the figure's origin is the hand, closing linearly to the bag at `touch_in_s` = 0, and the right wrist's reach is full weight at F = 1. Take-off, touch and up come from the sim's markers, not the client's own timers. The rise is the last 0.7 s of the sim's 1.0 s. Cosmetic flaws: P3-6. Take-off distance: P3-2.
7. **Measurements.** Verified in `live.rs` `absorb`: `margin_s` = |base − s| / `vmax` for appeals and forces, |goal − s| / `vmax` for tags. It is a distance at the out, not time lost, so the runner-start note's "90 close tag outs" were not shown to be close. On the Retrosheet comparison: P3-4.
8. **Spec and docs.** Mostly consistent with the code; the exceptions are in P3-1 (the cancel rule), P3-2 (1.8 m) and P3-3.

## Verification

All builds were in `sim/target/review-probes/q39/`: `tree/` is `git archive 0ec2704`, and `probe/` is the instrumented copy. All long runs went through the queue.

- **Job 130 (clean build of `0ec2704`):**
  - `cargo test --workspace --no-fail-fast`: **555 passed, 0 failed, 4 ignored** (`tests.txt`).
  - Release `sim-cli games --n 300 --seed 0` and `--seed 1000`: reports identical to the worker's `std-after-0.txt` and `std-after-1000.txt` in every line except timing. Runs 4.13 [4.46]. `rules warnings: {}`.
- **Job 131 (instrumented copy, `probe/`):**
  - `eprintln!` at the dive decision (with `m_back`, trapped, capped, distance, speed), the retouch decision, take-off, touch, up, and every out (with the runner's dive state).
  - The standard-run report is byte-identical to job 130's (seed 0 compared), so the prints do not change the game.
  - Analysed with `analyse.py`: the P2-1 table, the take-off distances (P3-2) and the cancellations (P3-1). Outs by dive state: 207 in the air, 113 in the run-in (dive chosen, still on his feet), none while down.
- **Job 133 (client, built fresh in `tree/client`, 74 s with `-MaxParallelActions=2`; build succeeded):**
  - `client/run.sh 7 -RenderOffscreen -BbSavedPlayCheck`: `BB_SAVED_PLAY_CHECK PASS commit=0ec2704 cases=39`.
  - `-BbDiveBackShots=2 -BbSpeed=6 -BbStagingLog`, seed 22: two dives (play 25, 0x102 into second; play 127, 0x004 into first), ten PNGs.
  - Seed 31: one dive (play 34, 0x101 into third), five PNGs. The run hit my 900 s timeout looking for a second dive.
- **Job 134:** `-BbSavedPlayCheckFile` on two old recorded saved plays (`seed5593-play18-20261122-225049.toml`, `seed51001-play128-20261120-002724.toml`): both `OPENED`, rc 0.
- **PNGs read** (`shots-22/diveback_00_1..5`, `01_1`, `01_3`, `01_5`; `shots-31/diveback_00_2`, `00_3`):
  - Take-off: upright, in his running pose.
  - In the air: laid out headfirst, both hands reaching.
  - At the touch: the right hand on the bag in all three.
  - Down, and then up through his hands and knees onto the bag.
  - The covering fielder overlaps him at every bag.
  - All three are retouches after a caught liner ("OUT, caught"), the commonest source (537 of 829).
- **Code checks:** `margin_s` in `absorb` (`live.rs` ~3440). Tick order: `holder_act` before `move_runners`. `BbDive` layout in the header and Rust. Retrosheet scripts `retro_third_gb.py` and `retro_second_gb.py` against the probe `zz_dive_back.rs`, and the probe's `tagback_gb` rows (50 from second, all 0–1 out, infielder first). `python3 bin/check-docs.py`: clean apart from this file not yet existing.

## Worker follow-up (worker-2, 2026-11-23)

- **P2-1 (fixed, `383e0d0`, spec `489e8fd`).**
  - He dives only on a margin between −`DIVE_SAVE_S` (0.4 s [E]) and `DIVE_CLOSE_S`. This applies in `runner_relook`, the capped path included, and in `retouch_dive` (`dive_can_change`).
  - Test: `a_capped_runner_beaten_by_a_second_brakes_in`.
  - Your count, redone with your prints and `analyse.py` on the fixed code over the standard 600 games: 829 → 552 dives, put out in the air 207 → 14, play ended in the air 8 → 2, capped runners under −1 s 175 → 7. The 7 are dives kept from an earlier close look (P3-1).
  - The outs are not unchanged. In the probe cohort, appeal outs went back from 133 to main's 141. The 8 saved before were retouches believed lost by more than 0.4 s whose hands got there. Dropping the lower bound on retouches alone would keep them (note §6; a choice for the Coordinator).
- **P2-2 (fixed, `383e0d0`, `95cf42b`).**
  - The dive is recorded at take-off. The record closes at the touch, at an out (tag or appeal) or at the play's end. Without a touch, the timeline has `Dive` and `DiveUp` and no `BaseTouch`, and the track holds until he is up.
  - Test: `a_dive_tagged_in_the_air_is_recorded`. The FFI timeline test checks touched and untouched dives.
  - The client draws the glide and the runner lying short of the bag. Headless check: seed 44, play 166 (the only dive back in that game, tagged in the air).
- **P3-1 (fixed).** A chosen dive stands while he keeps going back. Test: `a_chosen_dive_stands_while_he_keeps_going_back`. Run-ins cancelled inside braking distance: 6 → 0.
- **P3-2 (fixed).** `BbDive` carries `hand_from` and `hand_to` (the `Dive` and `DiveUp` points), and the client interpolates between them. The 0.4 m step at take-off is smoothed: the origin moves from his path point onto the hand over the first half of the glide. data-formats §5.4 is corrected.
- **P3-3 (fixed).** The test text in §6.4 now names a constructed close return. The tag-test comment is corrected. `retouch_dive`'s doc comment and §6.4 now say the throw is from where the catcher stands when the runner sees the call. Note §summary and §6 quantify the in-air outs.
- **P3-4 (fixed).** Note §3 and §6.4 list the per-fielder divergence (pitcher, first, short) and the infield-in confound as open, and say the `3XH` reading is unchecked.
- **P3-5 (fixed).** `bb_abi_version()` and `BB_ABI_VERSION` 2. A test ties the header's version to the library's. `FBbSim::Start` refuses a library of another version, or one without the function, and says why.
- **P3-6.** The 3,000-layout property tests now include runners diving back and down on a bag. The pose's blend starts 0.12 s before the sim's take-off, read from the frame ahead (`DiveBackLeadS`). The covering fielder's overlap is deferred with a line in TASKS §1.

## Resolution

Open.
