# Review: base swing build step 1 (export E1, the bat before the swing)

Reviewer: rev-swing-1 (Sonnet, medium effort). Date: 2026-11-24. Range `main..work/base-swing-step1` (2500aeb..0efd80e). Design: player-visuals §13 (step 1 of §13.8; measurements §13.10); PLAN D21–D23.

**Verdict.** No P1. E1 is read-only and cannot alter a game; the bat interpolation and the handover are correct against the sim's own swing, and the contact window is clear of the handover with 55 ms to spare in the worst case. One P2: the D22 check ("a swing and a take of the same pitch draw the same bat before t_L") is true by construction of the check, so it cannot catch the one realistic way D22 could break. The rest are P3.

## Findings

### P2-1. The D22 check shares its inputs with the thing it checks

`client/Source/BaseballClient/BbDirector.cpp:1939-1955` (`CheckBat`), with `:1879-1884` (`NoteBat`).

`CheckBat` builds the "same pitch as a take" by copying the swing's note and clearing `bSwing`, `TL`, `TS`, `TC` (`Withheld`). Everything the client's bat before t_L depends on, `N.TA`, `N.CatchT` and the stance, is copied unchanged. So `d22=0.0000 cm` (§13.10) is guaranteed whatever `TA` is, and the check cannot see the one realistic D22 breach: `TA` coming out different on a swing than it would on a take of the same pitch. `TA` is read from the ball's track, and on a pitch the bat hits that track ends at contact: `PitchPlaneT` (`:1667-1712`) then extrapolates from the last 30 ms (it also falls back to the constant 0.42 s when it returns −1). `TA` moves `Arrive = min(0.188, TA − 0.219)` and so the drawn hands on every pitch faster than the reference (TA < 0.407 s, about 5 mph over the median four-seam).

In practice the worker measured the extrapolation error at 0.0–0.1 ms (§13.10), which moves the hands by under 0.2 mm, so nothing visible is wrong today. The point is that the check would pass a real fault, for example `TA` taken from `swing_contact_s` on swings (a one-line change that moves `Arrive` by up to 100 ms on fast pitches), or the 0.42 s fallback being hit on swings only.

**Fix.** In `CheckBat`, take `Withheld.TA` (and `CatchT`) from an independent computation: for a swing that hit, compute the plane time from the track cut 30 ms earlier (the probe already does this), and fail if it differs from `N.TA` by more than 0.5 ms; for a swing that missed, compare with the same value read from the whole track. Or log `N.TA` and the `PitchPlaneT` of the track with the swing's last 30 ms removed. Then the 0.1 cm bound means something.

### P3-1. The new FFI test does not reach the load-to-swing boundary

`sim/crates/bb_ffi/src/lib.rs:1183`.

`the_swings_clocks_and_the_bat_between_its_samples` compares the interpolated bat with the swing's own bat only from `t_start + 0.0123` to `t_c + 0.11`. The three places where the interpolation rule could go wrong are outside that range: the stance sample to the first load sample (a hold when the gap is above `BAT_INTERP_MAX_GAP_S`), the load's last sample to the swing's first, and the stance-to-swing gap when there is no load. Making `BAT_INTERP_MAX_GAP_S` 0.5 s, which would blend the stance into the swing's first pose over a long gap, is a plausible fault the test would not see (mutation C below). The equality assertions on `swing_contact_s` compare the FFI with the same `sw.t_c` the encoder reads, so they test the plumbing (they would catch a 10 ms offset, mutation A) but not that `t_c` is the nominal contact; §13.10's probe covers that.

**Fix.** Extend the loop to start at `t_decide` (or at the first sample) and compare against the sim's own pre-swing bat (stance, then load straight line) with the same 1 mm and 0.2° bounds; add the two no-load swings' case with a gap above 7.5 ms and assert a hold.

### P3-2. Stale text: §13's heading and the document status still say "not yet built"

`docs/player-visuals.md:3` ("A redesign of the batter's stance and swing (§13, 2026-11-23) is specified and not yet built") and `:144` (the heading `## 13. … (design, 2026-11-23; not yet built)`, whose slug is linked from §9 `:78` and TASKS.md). Step 1 is built, and the §13 status line two lines below says so, so the heading and line 3 contradict it.

**Fix.** Line 3: "…is specified; build step 1 (the bat before the swing, export E1) is built, steps 2–5 are not." Rename the heading to "(design, 2026-11-23; step 1 of 5 built)" and update the two anchors (`python3 bin/check-docs.py` finds them).

### P3-3. The check puts no limit on the handover's hand speed, and the client run covers only 35 swings

`client/check-bat-step1.sh`, `BbDirector.cpp:2004` (`bFail` has no `SpeedHandover` term).

§13.10 reports start poses up to 48 cm from L and a handover speed of up to 8.5 m/s for the 28 cm gap seen in 35 swings. Closing 48 cm in 0.05 s with a smooth step is about 14 m/s at its peak, hands moving in the half-second before the swing at a speed a real hitter reaches only inside the swing. The continuity check (2 cm / 2° of the motion carried on) passes this by design, so it is not a defect of the check; but the one number that says how a large handover looks is logged and not judged, and the largest gaps (the tail of 5,019 swings) were not run in the client. The owner will see these. **Fix.** Fail the check, or at least print a line in the script's summary, when `speed_handover` exceeds a stated figure (say 10 m/s) and report the count over a longer run (more seeds), or accept it and say so in §13.10 with the 48 cm figure's speed.

### P3-4. Nit: dead locals in `InferContactT`

`BbDirector.cpp:1771,1777`: `X0, X1, X2` are declared and then discarded with `(void)`. Delete them.

## What was checked and found sound

1. **E1 cannot alter a game.** Over `main..HEAD`, `bb_game` changed in one place: `timeline.rs`, comments plus the `swing_contact_s` field in `Telemetry` (`rec.swing.map_or(-1.0, |sw| sw.t_c as f32)`). Nothing in `bb_world`, `bb_agents`, `bb_rules` or `pa.rs` changed; the sim's logic is untouched since 2500aeb (`e56748b` touched the probe example only). The schema field is appended (`id: 23`, default −1.0; generated `VT_SWING_CONTACT_S = 50` = 4 + 2·23, default `Some(-1.0)`), so an older timeline reads −1 and `an_old_saved_play_has_no_swing_contact_time` reads the tracked seed-51001 play. `BbPlayInfo` gained two `f32` after `_pad3[2]`: both sides are 4-aligned with no new padding, and the header struct and the Rust `repr(C)` struct agree field for field; `BB_ABI_VERSION` is 3 in both, and `BbSim.cpp:47` refuses a mismatch. The digest: job 172 (base `82751d0`) and job 184 (after E1) print the same four digests (seeds 1, 2, 3, 42: `bc3d1a76…`, `87993b60…`, `10ed6ec1…`, `4a5141ce…`), 184 ending `IDENTICAL`; job 175 (`cargo test --workspace`) all ok. I did not rerun them. Invariant 5: no transcendental math, `HashMap`, `mul_add` in what was added in `timeline.rs` or the FFI interpolation (`sqrt` only; the FFI is presentation and outside the sim's determinism contract in any case).
2. **Interpolation (`bb_timeline_sample`, `lib.rs:572-598`).** At the track's end `i = n − 1` and `i + 1 < n` is false, so no extrapolation; at a sample time `t > k.get(i).t()` is false, so the sample is returned exactly (same as before). The knob is a straight line, the direction a normalized blend (`len > 1e-6` guard, falls back to `a0`); samples 5 ms apart are at most about 10° apart, so the blend is within the test's 0.2°. A checked swing's stop is a run of equal samples, so interpolation is a hold. A left-hander is the mirror (the blend is on world vectors). The `n == 1` take, the `+0.25 s` hold and `has_bat` are unchanged, and the old-play test confirms it. The stance sample to the first load sample, and any gap above 7.5 ms, is a hold, as the old code. The new FFI test (seed 11, bounds 0.15 cm and 0.2°) passed in job 175.
3. **D22, read from the code, not the 0.0000.** Before `TL` on a swing, `DrawnBat` returns `LoadBat(T, N.TA, !N.bSwing, N.CatchT, …)`; the take's relaxation begins at `CatchT + 0.2 s`, always later than `TL`, so the `bTake` flag changes nothing there. `BatSwingW`, `BatStride`, `TurnDeg` start at `StartT = TL`. The sim's stance and load samples are not drawn before `TL`. The only input that could differ is `TA` (P2-1). Old saved plays (no E1) take the same path: `DrawnBat` never reads `TC`, which only the log uses, and `TS` falls back to `TL` when the marker is missing.
4. **The handover (Q4).** `HandoverWeight` ends at `max(TS, TL + 0.05)`. `LOAD_S` is 0.05 s, so `TL ≥ TS − 0.05` and the end is in `[TS, TS + 0.05]`. Nominal contact is at least `TS + T_swing` ≥ `TS + 0.130` (T_swing runs 0.130–0.174 by `QuickHands`, `attributes.rs:76`; `player-visuals.md:194`); the actual contact is within 0.025 s of it. So in the worst case (a decision just before the swing, the load cut to nothing, the shortest swing) the drawn bat is the sim's from `TS + 0.05`, which is 0.08 s before nominal and **at least 0.055 s before actual contact**: never inside the contact window or at contact. For 0.05 s into the swing (at most) the drawn bat is a blend of the load pose and the sim's moving bat; no ball interaction there, so Invariant 4 holds (the bat at contact is the sim's).
5. **Docs.** `docs/data-formats.md` (E1 paragraph and ABI line), the ledger line ("No effect on games"-type, with the digest claim), AGENTS' `-BbBatShots` and §13.8/§13.10 agree with the code. Apart from P3-2 I found nothing stale. `bin/check-docs.py` reports only this review's own path (not yet written when it ran).

## Verification

- Read in full: `git diff main..HEAD` for `timeline.rs`, `timeline.fbs`, the generated telemetry code, `bb_ffi/src/lib.rs`, `bb_ffi.h`, `BbPose.{h,cpp}`, `BbDirector.h`, and the new parts of `BbDirector.cpp` (`NoteBat`, `PitchPlaneT`, `InferContactT`, `DrawnBat`, `CheckBat`, `PoseFigure` and the bat part of `Tick`); the player-visuals, data-formats, ledger, AGENTS and TASKS diffs.
- Job logs read, not rerun (queue labels `swing-1 step1`): 172 and 184 (digests, identical), 175 (`cargo test --workspace`, all ok). `git merge-base` = `82751d0`, the base the worker cites.
- `python3 bin/check-docs.py`: only the not-yet-written path of this review.
- **Mutations** (copies of `sim/` in the scratchpad, built with `CARGO_TARGET_DIR=sim/target/review-probes`, run through the queue as job 210, `cargo test --release -p bb_ffi the_swings_clocks`): A, `swing_contact_s` written 10 ms late; B, the interpolated knob offset by 5 cm in x; C, `BAT_INTERP_MAX_GAP_S` 0.5 s. **Expected** (not yet seen): A fails on the `swing_contact_s` equality, B fails on `worst_cm`, C passes (P3-1). **Results: see the Mutation results note below.** The client-side faults (P2-1: `TA` from the swing's contact) were not run: a client rebuild and the bat check need about 25 minutes of the shared queue; the argument is from the code (the `Withheld` copy shares `TA`).

### Mutation results

Run by the author of the fixes (the reviewer's job 210 failed on the unquoted space in the repository path): copies of `sim/` in the scratchpad, `cargo test --release -p bb_ffi` on the two E1 tests, the new one included (queue job 245, log `sim/target/swing1-mutations.txt`).

| Mutation | Result |
| :-- | :-- |
| A: `swing_contact_s` written 10 ms late | fails: `the_swings_clocks_and_the_bat_between_its_samples` (the `swing_contact_s` equality) |
| B: the interpolated knob 5 cm off in x | fails both: 5.037 cm off the swing's own, and 5.000 cm off the sim's bat at 0.2601 s in the new test |
| C: `BAT_INTERP_MAX_GAP_S` 0.5 s | passes the old test (as the review expected), **fails the new one**: with no load the bat blends from the stance toward the swing's start over the whole pitch (0.254 cm and 0.64° off at 3.4 ms) |

## Resolution

Resolved 2026-11-24 on `work/base-swing-step1` (merged with main `2b3c1eb`).

- **P2-1: fixed.** `CheckBat` no longer copies `TA` and `CatchT` into the withheld take: it works them out again (`PitchPlaneT` gained a `CutAt` argument: a ball hit, from the track cut 30 ms before contact; a miss, from the track cut at the swing's nominal contact; the catch from the call time) and fails when the swing's `TA` is more than 0.5 ms away (`ta_err` in `BB_BATCHECK`). A pitch that never reaches the plane on its track (in the dirt) has no `TA` to compare. **Planted fault** (`TA` taken from the swing's contact on swings, applied to the working tree and restored, built twice): the check fails all 18 swings of seed 7, `ta_err` 1.4–71 ms and `d22` up to 6.7 cm; restored, seeds 6 and 7 pass (0 FAIL, 35 swings). The new check also found a real bug the old one could not: `PitchPlaneT`'s scan stopped a step before contact, so a ball hit just past the plane fell back on 0.42 s (26 of 398 swings). Fixed in the same commit series (`PitchPlaneT`'s loop now ends at contact).
- **P3-1: fixed.** New test `the_bat_before_and_through_the_swing_is_the_sims_own` compares the drawn bat with the sim's own over the whole track (stance, the load's straight line, the swing) at 1.7 ms steps and at both sides of the stance-to-load and load-to-swing boundaries, for 40 swings each with the decision as it was, with no load (`t_decide` = the swing's start) and with the load cut to 20 ms. Mutations A, B and C now each fail something (table above); C failed nothing before.
- **P3-2: fixed.** `docs/player-visuals.md` line 3 and the §13 heading ("step 1 of 5 built"); the three anchors (PLAN, TASKS, §9) renamed.
- **P3-3: judged and printed, not redesigned.** `client/check-bat-step1.sh` prints the handover's hand speed over all the swings it checked (median, p95, worst, how many above 10 m/s, the largest gap) and takes `BB_SEEDS`, `BB_SPEED`, `BB_HANDS`, `BB_MIN_SWINGS` for a long run without the hands comparison. 406 swings: worst 13.5 m/s, 17 (4.2%) above 10 m/s, three of them (gaps 38–45 cm) 2.06–2.34 cm from the carried-on motion at the handover's end (limit 2 cm), reported by the script as failures. §13.10 has the figures and the judgment; the handover stays as it is until Q44.
- **P3-4: fixed.** The dead `X0, X1, X2` are gone.
