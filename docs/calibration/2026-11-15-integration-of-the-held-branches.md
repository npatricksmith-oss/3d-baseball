# The three held branches together (2026-11-15)

Branch `integrate/held`: main `1a06d22` (the runner reads merged) plus `work/collision-q` (sourced q 0.215, the inboard barrel falloff, partial reach g = 0.6), `work/outfield-read` (the spin belief refit for swing v2, the closing read, `ReadSkill`) and `work/wall-braking` (`BRAKE` 4.75 m/s², wall-contact speed 2 m/s, one over-the-wall reach), merged by the lead, and on top of them two steps of this job:

1. **Tests** (`f6cf457`). Every branch moves games, so tests that are pinned to a play are found again:
   - **`isolation_seed_0_a_private_cutoff_job_moves_nobody`** (was `…_play_0_…`). The review's snapshot (play 0, t = 1.998 s) no longer arises. It is now the first `update_relay` in seed 0 with the second baseman idle and the catcher backing up: play 3, t = 0.946 s. A negative control checks the snapshot still tests something: with the old gate restored, the catcher leaks to Cover(4).
   - **`seed_23080_play_8_is_caught`** is **ignored, with its reason**. It is not a pinned-play artefact (§3).
2. **The fielders' typical-spin belief, refit on this contact population** (`5011ab7`; the outfield-read review's P3-1; agent-models §5.1). The fit is the same as before: `examples/batted_spin.rs`, 48 games, median backspin by 10° launch-angle bin.

   | Angle | −25° | −15° | −5° | 5° | 15° | 25° | 35° | 45° | 55° |
   |---|---|---|---|---|---|---|---|---|---|
   | Before (rpm) | −4,856 | −3,077 | −1,783 | −477 | 606 | 1,916 | 3,013 | 3,819 | 4,558 |
   | After (rpm) | −4,699 | −2,998 | −1,732 | −577 | 688 | 1,909 | 2,949 | 3,770 | 4,504 |

   The medians moved by under 3% at 25°–55°, so the median by angle was not stale. The staleness test (`the_typical_spin_belief_follows_the_contact_model`) changed in two ways:
   - It now runs on 24 games outside the fitting sample (seeds 100–123). Its 6-game sample had failed on size: 28 balls in the 55° bin.
   - It adds groups by exit speed: under and over 85 mph, each at 25–35°, 35–45° and 45–55°.

   **Exit speed is where the belief is off.** It is by angle alone. Contact under 85 mph carries 11–29% less backspin than it believes, and contact over 85 mph 5–16% more (by median, in three samples of 24–48 games). The test therefore allows 35% per group, which still catches a table as stale as 2026-11-12's (42–72% high). A belief by angle and exit speed is open, not built.

   A no-draw landing-bias check was tried as the staleness test and rejected. Two seconds before landing the read has closed: a table 60% too high moved the mean miss there by under 0.5 m. Three seconds out, the refit table reads flies 0.5–1.3 m deep on average and a stale one 1.0–2.6 m.

No other model, knob or tolerance was changed. Main's row is the ledger's runner-reads row after the Codex recheck (main `3f8ee60`, 5.05 [4.97]); `1a06d22` adds no sim change to it. Real MLB is the 2024 season from the ledger's §1 sources.

## 1. Headline (300 games, seeds 0–299 [the `--seed 1000` set])

| | main | this branch | real MLB |
|---|---|---|---|
| runs / team-game | 5.05 [4.97] | **3.88 [3.92]** | 4.39 |
| K% | 21.9 [21.6] | 22.3 [22.3] | 22.58 |
| BB% | 8.2 [8.5] | 8.3 [8.8] | 8.18 |
| HR / PA | 2.00% [1.97%] | **2.57% [2.65%]** | 2.99% |
| reached on BIP | 42.1% [42.1%] | 34.7% [34.2%] | |
| GB hits | .331 [.324] | .291 [.295] | about .245 |
| LD hits | .758 [.762] | .668 [.665] | about .68 |
| FB hits | .225 [.223] | **.112 [.116]** | about .13 |
| PU hits | .035 [.031] | **.028 [.018]** | about .02 |
| PA / team-game, pitches / game | 40.0, 250 [40.1, 250] | 38.4, 243 [38.8, 244] | |
| rules warnings | none | none | |
| routine flies caught (`fly_misses 0 300`) | not run on this main (.824 on `dc1ce82`) | **.956** (25–35° .926, 35–50° .962, 50°+ .979; 1,809 flies) | 90–99% by ability (owner) |
| flies in the outfield that fall | | .106 | |
| batter tagged out after a hit (`runner_outs_probe 300 0`) | 113 | 98 | |
| other runners tagged out, hit plays | 24 | 41 | |
| runners tagged out, fielder's-choice plays | 156 | 116 | |
| exit speed, fair balls, seeds 0–299 (`poor_contact`): mean / under 80 mph / hard-hit / 105+ | 89.6 / 19.6% / 39.1% / 5.5% (main `4fe04d7`, pooled) | 89.1 / 23.2% / 39.9% / 8.2% | 88.6 / 25.0% (competitive 23.1) / 40.3% / 9.9% |

**Sources of the main column.**
- The runner-outs counts are the ledger's runner-reads row.
- The exit-speed row for main is from the collision-q note, measured before the runner reads. The runner reads do not touch contact.

**This branch against the outfield read alone.** On its own (on main `dc1ce82`), the outfield read gave:

| | outfield read alone | this branch |
|---|---|---|
| runs | 3.32 [3.42] | 3.88 [3.92] |
| FB hits | .099 [.102] | .112 [.116] |
| routine flies caught | .967 | .956 |

Collision-q adds hard contact and home runs. The runner reads add runs per hit. Wall braking costs the outfield a little range.

## 2. The read's no-draw bias, by launch angle (`fly_misses 0 300`)

The responsible outfielder on routine flies. Depth is along the line from home to the landing point; + = deeper. Each cell is mean / sd in metres, at 2.0, 1.0 and 0.5 s before landing.

| | 25–35° | 35–50° | 50°+ |
|---|---|---|---|
| depth, his read | +0.44/2.90, +0.08/0.86, +0.01/0.20 | +0.45/2.39, +0.19/0.71, +0.04/0.30 | +0.44/2.05, +0.12/0.63, +0.01/0.41 |
| lateral, his read | +0.11/2.50, 0.00/0.81, 0.00/0.22 | +0.03/2.15, −0.01/0.76, 0.00/0.25 | +0.01/2.05, −0.01/0.73, −0.01/0.26 |
| depth with no draws | +0.32/0.81, 0.00/0.19, −0.02/0.06 | +0.34/0.40, +0.10/0.16, +0.03/0.05 | +0.24/0.40, +0.07/0.15, +0.02/0.04 |

The read has no meaningful systematic error left by 1 s before landing. Two seconds out it is a third of a metre deep at every angle. On the outfield-read branch it was −0.1 to +0.3 m.

**Causes of the 79 routine flies not caught:**

| Cause | Flies |
|---|---|
| Read good late, too far to make up | 35 |
| In dive range, no dive | 35 |
| Reached, not caught | 6 |
| Left to a teammate, who missed | 2 |
| Late read error | 1 |

## 3. Seed 23080 play 8, the owner's play

Play 8 is now a harder ball: 98.5 mph at 45°, 6.5 s in the air, landing at (−35.0, 93.8). The left fielder starts 64 ft away, which is still routine. **It falls.** Traced with `read_track 23080 8 7`:

- He is the backup until 2.4 s.
- His read then drifts deep: +9 m along the line at 3.6–4.0 s.
- He runs out to 98 m deep, braking from 4.2 s, and turns back at about 5.4 s. He arrives 2 m from the ball at 6.8 s, with his read now exact.

This is the outfield-read review's P2-1 (the read keeps no track of what it has seen), deferred to the tracking-read job. It is joined by `work/wall-braking`'s slower braking. With `BRAKE` back at 6 m/s² (a temporary probe, not kept), the same ball with the same reads is caught at the edge of his reach. At 4.75 m/s² the turn costs him the few tenths of a metre.

The test is ignored, not inverted: "not caught" is a defect, not an expectation. It comes back with the tracking read. **The owner's own play is dropped on the merge candidate.**

## 4. Pop-ups

Pop-up hits are .028 [.018] here: about .023 pooled, against about .02 real. On main they are .035 [.031]; with wall braking alone, .046 [.057]. The wall-braking review's mechanism was a chaser at full speed on a late-corrected read who overshoots. The likely reason is the closing read, which corrects the read earlier so that the slower braking matters less on a pop-up. That is not traced. Pop-up hits are not well above .02, so no play is traced here. The "arrive early and under control" follow-up still stands for the plays the wall-braking review traced; this measurement does not show whether those two plays are still lost.

## 5. What is still off

Runs are 3.88 [3.92] against MLB's 4.39, about half a run low. The run total moves this way because the outfield read takes away more hits than collision-q and the runner reads add. No knob was moved toward it.

The rows that make it up:

- **Fly-ball hits** are .112 [.116] against about .13. The cause is unresolved: it is not shown to be the read or the legs. The outfield-read review's P2-2 is open: measure movement and the read against matched catch-probability bins.
- **Line drives** are at MLB's about .68 (.668 [.665]).
- **Grounder hits** are .291 [.295] against about .245, still high, the fielding gap the ledger has carried for days.
- **Reached on BIP** is 34.7% [34.2%].
- **HR/PA** is 2.57% [2.65%] against 2.99%, the collision-q branch's known shortfall (105+ and barrels short; the launch angle of balls hit 100+ mph too low).
- **K% and BB%** are at MLB's.

Within the outfield, about 4% of routine flies still fall, against the owner's 90–99% band at grade 50. 35 of the 79 are dives not taken at the edge of range; 35 are men who read the ball well late but started from too far. Some of those are the mid-flight deep drift (§3), which a slower `BRAKE` now makes costlier to recover from.

The fielders' spin belief takes no account of exit speed: 11–29% too much backspin on soft contact. The landing bias that leaves is small by 2 s before landing (§2), so it is recorded, not fixed.
