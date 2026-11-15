# Fielder communication, fourth review (Opus, standing in for Codex)

**No P1. Merge.** Two P2 findings, both about the isolation guarantee rather than wrong results on this branch: one is a leak older than the branch that its harness cannot see, the other is a set of gaps in that harness. They should be recorded and fixed next, but they do not block the merge.

## What was reviewed

- Branch `work/live-comm` at `5cfb10e`. Range `ae440b9..5cfb10e`, code `74ea51b`, base main `57f12b7`.
- Code: `sim/crates/bb_game/src/live.rs`, covering:
  - the recovery order (`public_rank` 1739, `public_path` 1755, `fallback_chaser` 1767);
  - the partner rule (`partner_fields_it` 1858);
  - `hear` 1995 and its fallback at 2037;
  - the cutoff and relay changes (`update_relay` 2332);
  - the isolation harness (hook 4004-4064, tests 6639-6800).
- Docs: `live/holder.rs`, spec `docs/agent-models.md` §5.7, the calibration note, the ledger row, and the three earlier Codex reviews with their Resolutions.
- Method: probes ran in a scratch copy of `5cfb10e` under `sim/target/opus4/` (instrumentation patch plus probe tests). Nothing tracked was edited.

## Findings

### P2-1: The holder's private throw choice moves a receiver in the same tick, and the harness exempts the holder

`live.rs:3718-3720`:

```rust
if race.direct < race.receiver_at - 0.05 {
    self.f[receiver].target = tgt;
    return;
}
```

**The leak.** Before releasing the ball, the holder privately picks a base and a receiver (`holder_act` 3565 → `receiver_for` 3816). If the receiver is not there yet, this code sets that teammate's target to the bag, in the same tick. No call is published and no `CALL_HEARD_S` passes. The receiver's own job (role, `known_job`) does not ask for it. This is the recurring defect class: one man's private decision re-works another man's movement.

**It is older than the branch.** The same lines are at `57f12b7:live.rs:3126`.

**Why the harness misses it.** At `live.rs:6721` the harness skips the holder in `holder_act`. It never flips the holder's private inputs, so this path is outside its coverage. It also contradicts the branch's statement that no man's job is now re-worked by another's private state.

**Probe.** `opus_holder_redirects` instruments line 3718 and logs every redirect that moves the receiver's target by more than 1 m. Over seeds 0-49 there were 13 redirects in 5 plays, all to second base (0, 38.8):

| Play | Holder | Receiver | Own role (known role) | Move |
|---|---|---|---|---|
| 1/195 | f7 | f6 | Backup (known Backup) | 17.8 m |
| 8/121 | f7 | f3 | Relay(3) (known Cover(2)) | 9.6 m, re-imposed on 8 consecutive ticks; then f5, Idle, 8.1 m |
| 27/193 | f8 | f6 | Backup | 10.5 m |
| 46/224 | f5 | f6 | Backup | 10.8 m |
| 49/78 | f7 | f6 | Backup | 16.9 m |

In four of the five plays, a man doing a public Backup job is pulled 10-18 m to a bag the moment the holder decides, before anyone could have heard him.

**Impact.** The frequency is low, about one play in ten games. I did not measure whether an out changes.

**Fix.** Make the holder's choice a call: publish a `Shown` "throw to b" and have the receiver redirect only from `known` once `CALL_HEARD_S` has passed. The post-release redirect at 3746 can stay, because a thrown ball is public. Extend the harness for `holder_act` too: flip the holder's private inputs (belief, error, planned base) and assert that no non-holder target changes before a throw is in the air.

### P2-2: The harness covers less of the class than its name says

The committed harness (`isolation_no_private_state_reaches_a_teammate_in_real_games`, 6752) does what it claims for what it covers. With the P1-1 gate restored in the scratch copy (`OPUS_REVERT`), it reports 40 leaks in seeds 0-4. The first is at seed 0, play 2, t 2.9136, `update_relay`, where f1's backup changes. Its gaps:

1. **The first read of a new loose ball is not hooked.** `set_free` (2992) calls `self.assign()` at 3010, outside `simulate_live_inner`'s eight hooks. Every deflection, bobble and overthrow gets an unchecked first assignment. The scratch patch adds `iso_check(self, "assign_new")` there; it is a one-line hook.
2. **The flips are narrow.** `iso_flips` (6659) flips Idle, Chase, Cover(1), Cover(2), Relay(4) and Backup. It never flips Cover(3), Cover(4), Relay(1-3), `backs` or `err`, the very fields the cutoff and backup logic branch on, and P1-1 in the third review was a backup leak.
3. **The comparison is narrow.**
   - `iso_state` (6639) compares role, target, backs, peel, call, read time and intercept, but not `next_plan`, belief, `shown`, or the runners.
   - `iso_ball` compares only the ball.
   - A private flip that reached a teammate's `next_plan` or `shown` would be caught only later, if at all.
4. **The sampling is sparse on rare steps.** With the default stride of 200, `recall` (once a second) and `update_relay` retirements are sampled a handful of times per game.
5. **The holder is exempt.** See P2-1.

**Fix.** Hook `set_free`'s `assign`. Add the missing roles, `backs` and `err` to `iso_flips`. Compare full `Fielder` state except the flipped man, and include runners. Run the rare steps at stride 1, since they are cheap per call even if the full run is not. I wrote all of this in the scratch patch (`sim/target/opus4/patch.py`), but it did not finish within the time limit (see Verification), so the widened check has no result either way.

### P3-1: A pop-up near the mound ties on the scan grid, and the tie goes to the pitcher

`fallback_chaser` (1771) breaks ties to the lower index. On the recovery probe, pop-ups give exact ties:

- Pop-up near the mound (EV 22, LA 75°): P, C and 1B all rank 3.53 s.
- Pop-up behind 1B: P, 1B and 2B all rank 3.93 s.

The pitcher is sent both times. Real infields call the pitcher off pop-ups. This is believable but wrong in spirit, and it only matters once everyone is known not going.

**Fix.** Break ties by `CALL_PRIORITY`, or by the margin from the man's position to the landing point, before the index.

### P3-2: The second-base worst gap is 2.067 s, above the documented maximum

My coverage run over seeds 0-49 reproduces the counts. The second-base maximum, however, is 2.067 s at seed 3, play 140, above the 1.333 s cited from the third review.

- What happened: a fly at LA 47°. 2B, SS and CF all chase from the first read, and LF catches it, so no out is lost.
- Next worst: 38/18 (1.333 s) and 49/199 (1.317 s, a pop at LA 71°, caught).
- Most second-base gaps are both middle infielders chasing at the first read.

**Fix.** Correct the maximum in the note. Consider making the partner rule also stop the second middle infielder when an outfielder is chasing a ball in the air.

### P3-3: The note should state that the paired replay shows no fly-ball defense effect

From the author's `sim/target/fourth/paired.log`:

- On the 1,129 paired flies, catches went 14 lost against 15 gained, and hits 383 on main against 382 on the branch (13 branch-only, 14 main-only).
- The paired branch side reproduces all 50 real branch digests.
- The main side's `live.rs`, `holder.rs` and `choice_trace.rs` are byte-identical to `57f12b7`.

So the defense is neutral on flies. The standard-run movement (seed 0 up from 4.31 to 4.46, seed 1000 down from 4.68 to 4.58) is in opposite directions on the two seeds and is not explained by fly-ball defense.

**Fix.** In the note and the ledger row, label it as divergence or noise, not a defense effect.

### P3-4: The public-rank cache key omits velocity

`rank_cache` (1305, read 1741, written 1748) is keyed on `(t, ball_gen, positions)`, but the intercept it caches depends on each man's velocity too. Today this is digest-neutral:

- velocities change only in `move_fielders`, after every decision step in a tick;
- `t` advances every tick.

Reproduced digests match the claims (seed 42 `c3f7a99c…bfb4a`, repeated twice). However, a future decision step placed after a velocity change, or a second query at the same `t`, would read stale ranks without anything failing.

**Fix.** Add velocities to the key, or add a `debug_assert` that recomputes and compares in test builds.

## Answers to the lead's questions

1. **Isolation.** The harness is real: it catches the reverted gate (40 leaks). It does not cover the holder, new-ball assignment, or several roles and fields (P2-1, P2-2). P2-1 is a live leak of the same class, outside the harness and older than this branch.
2. **Recovery order.** It reads only public inputs: the current ball state, typical spin, no bounce noise, and positions. That ball state is a documented simplification. The order is acyclic: one argmin over a total order (rank, then index), with known Cover(1) excluded, so first base stays protected. The probes sent sensible men:
   - slow roller toward 3B: P (1.53 s);
   - roller down the 1B line with covers: C (P and 1B excluded as Cover(1));
   - dribbler: C;
   - grounders down the lines: LF and RF;
   - balls past the outfielders and deep gaps: CF;
   - RF line drive: RF;
   - seed 2 play 140 contact: CF (3.85 s); played to the end on the fixture, CF secures it at 4.317 s.

   Exceptions: pop-ups (P3-1), and a shallow fly over SS where the public typical-spin path sends SS while the true zero-spin path favours LF. The second is the stated limit of a public model, not a defect.
3. **Second base, the partner rule and the table.** Reproduced over seeds 0-49:
   - Gaps over 0.3 s: first 3, second 15, third 31. Nobody chasing: 0. Double cover: 0. Repeats of all nine: 0.
   - The three first-base gaps:
     - 37/34: 1.017 s, a grounder whose throw got away; the batter reached second.
     - 41/222: 0.75 s, a foul pop at LA 56°; caught for a foul out.
     - 26/10: 0.417 s, 1B out as Relay(4) on a loose ball after a throw; batter to second, runner first to third.
   - No out is lost in any of them, since none had a play at first available. The second-base maximum is above the documented one (P3-2).
4. **Numbers.** The branch standard run is reproduced exactly:
   - Seed 0: 4.46 runs, FB .222, K 21.5%, BB 8.1%, HR/PA 1.92%, rules warnings {}.
   - Seed 1000: 4.58, FB .231, warnings {}.

   I spot-checked the paired replay from the author's log, which shows 14 catches lost against 15 gained (P3-3). Recording the 11/1 and 4/280 mechanisms rather than fixing them is acceptable for a merge. 11/1 is CF's own misread tier while RF is also going; 4/280 is LF's backup depth. Both are mistakes a real outfield makes, rare, and not caused by leaked information.
5. **Invariants.** In `57f12b7..5cfb10e`:
   - No new `HashMap`/`HashSet`.
   - No direct transcendentals outside tests (`to_radians` is test-only).
   - No new RNG and no outcome rolls.
   - Runners read only public fielder state.
   - The same seed gives the same digest on two runs (seeds 1, 2, 3 and 42).
   - The rank cache is digest-neutral by ordering, but its key is fragile (P3-4). I did not run a cache-bypass digest comparison.

## Verification

- `cargo test --workspace --release` on `5cfb10e`: 359 passed, 0 failed, 3 ignored.
- Standard runs `sim-cli games --n 300 --seed 0` and `--seed 1000`: they match the ledger, as listed under question 4.
- Digests: seed 42 matches the claim, and seeds 1, 2, 3 and 42 repeat identically.
- Harness against the reverted gate (`OPUS_REVERT`, seeds 0-4): 40 leaks, matching the claim.
- Holder redirects (`opus_holder_redirects`, seeds 0-49, 23 s): 13 redirects in 5 plays (P2-1).
- Recovery probes: `opus_recovery_scenarios` (14 ground and 7 air scenarios) and `opus_full_plays` (2/140).
- Coverage: `BB_COVER_TRACE=1` over seeds 0-49 with `analysis/coverage_gaps.py`, reproducing the counts in question 3.
- Paired replay: spot-checked from the author's `sim/target/fourth/paired.log` and sources, not re-run.
- **Did not finish: the widened isolation harness.** It was run with the extra roles, `backs`/`err`/`next_plan` flips, full-state comparison and the `set_free` hook.
  - The first attempt also ran the rare steps at stride 1, over seeds 0-9. The lead stopped it after 1 h 46 min with no output.
  - A narrowed rerun (seeds 0-9, stride 200, no stride-1 steps) hit a 9-minute cap. The harness reports only at the end, so there is no partial result.
  - The widened check is therefore neither passed nor failed. It is what P2-2 asks the author to add.
- Not run: a cache-bypass digest comparison, and a measurement of whether P2-1's redirects change an out.
- All probes are under `sim/target/opus4/`. `git status` shows only this file.

## Resolution

Every finding is fixed except the behaviour change P3-2 suggests, which is deferred (worker session 2026-11-14; details and numbers are in the Codex fourth review's Resolution, which was the merge gate).

- **P2-1 (the holder's private choice moves a receiver; the harness exempts him): fixed, `12cdb26`** (Codex P1-1). As you suggested, the holder's choice is now a call: the man goes to the bag once he hears it, `CALL_HEARD_S` later, and only while that holder still has the ball and still calls him (`Fielder::asks`, `Shown::asks`, `heed_requests`). The release-time redirect after `throw_to` stays: a thrown ball is public. The harness checks the holder at his own step up to a release, flipping his set-up clocks and his call. The redirects you counted change no digest on seeds 0–49 and leave the standard run unchanged: the called man moves 0.3 s later, and no event depended on it in those plays.
- **P2-2 (the harness covers less than its name): fixed, `3b7cb38`** (Codex P2-1): `set_free`'s first read is hooked, as are the replans, close and dive reads, a throw's read and backup; every role, `backs`, `err` and the planning clocks are flipped, along with valid alternative reads; every other man's whole state is compared, `next_plan`, belief and `shown` included, with the runners and the ball. Rare steps and job-changing calls are checked at every call. It completes: seeds 0–1 in the suite, 0–49 in an ignored test (MANY_SEEDS_RESULT). Your restored gate, as a committed negative control, is found: 227 violations in seed 0's first 30 plays; with Codex's settings 4,724 at 169 snapshots.
- **P3-1 (a mound pop-up ties and goes to the pitcher): fixed, `3b7cb38`.** A dead heat in the shared order now goes by the call priority everyone knows: an outfielder over an infielder, then `CALL_PRIORITY` [E] (SS, 2B, 3B, 1B, C, P), then the lower position, so the pitcher is last, as on a field, where he is called off pop-ups. The same order breaks ties in which of two chasers keeps going (`publicly_before`). Test `a_tie_for_a_pop_up_nobody_took_does_not_go_to_the_pitcher` (your probe: the pitcher ties for first and is not sent). It changed 6 of 50 games.
- **P3-2 (second base's worst gap above the documented maximum): prose fixed.** The note gives both: 2.067 s at 3/140 on `74ea51b` (that game no longer occurs) and 1.333 s at 38/18 now. **Deferred:** stopping the second middle infielder when an outfielder is chasing a ball in the air. It changes how the infield plays a fly, so it is a feel decision, and the owner-approved motion-onset rule (a man seen breaking for the ball is known before his call is heard; agent-models §5.7) bears on the same cases. It should be measured after that rule is built.
- **P3-3 (the paired replay shows no fly-ball defense effect): fixed in the note and ledger.** Re-run on `3b7cb38`: 1,116 flies, 12 catches lost and 15 gained, hits 377 on main against 375. The note says the paired replay shows no fly-ball defense effect and that the sample cannot prove the defenses equal; the ledger's fly-ball lead is marked weak.
- **P3-4 (the rank cache key omits velocity): fixed, `3b7cb38`.** Velocities are part of the key. Test `the_shared_order_follows_a_change_of_velocity` (a change of velocity alone in one tick gets a fresh rank; it read the cache before). It changes no game: `3b7cb38` built with only the old tie key restored gives the same 50 digests (seeds 0–49) as `2969aee`, which has the old key; the 6 games that differ between the two are the tie order's.
