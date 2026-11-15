# Outfielders miss routine fly balls: the read (2026-11-15)

Branch `work/outfield-read`, from main `dc1ce82`. Owner's guidance (TASKS.md, "Outfielders miss
easy-looking fly balls"): a high ball *is* harder to judge, keep that; but a real outfielder
makes a play like seed 23080 play 8 (48 ft to cover, 6.2 s of hang time) 90–99% of the time,
depending on his fielding ability. Fielding errors (A17) are a separate, later feature.

## 1. Diagnosis (before any model change)

**Tool.** `cargo run --release -p bb_game --example fly_misses -- 0 300 list` (new; read-only).
It turns on `live::read_trace`, a per-tick record of every fielder's belief and motion while a
batted ball is on its first flight; the record is written only, never read by a decision, and is
off unless a diagnostic switches it on for its own thread (the games' digests do not change).

**Routine.** A fly is a fair ball launched at 25° or more that comes down in the park beyond
the infield dirt (47.4 m), not within 3 m of a wall, and not off a wall first. It is *routine*
when the outfielder nearest the landing point at contact needs to cover at most 0.60 of what an
average real outfielder covers in the opportunity time (Statcast's clock: pitch release to
landing): 33 ft in the first 3 s (the middle of the 2022 single-season range 29–38 ft,
FanGraphs, research note §2) and 27 ft/s after (Statcast's average sprint speed). The 0.60 is
Statcast's 99% example (Renfroe: 39 ft in 4.2 s of opportunity time, 99%, FanGraphs "Early
insights from Statcast's outfield catch probability metrics"), so the set is plays Statcast
rates about 95–100%, i.e. 1-star or easier. Savant's own table (distance × opportunity time) is
rendered client-side and could not be read; the boundary is therefore anchored on one published
example, not fitted. Seed 23080 play 8 needs 49 ft of 129 (0.38): routine.

**Seeds 0–299 (main `dc1ce82`):**

| | Routine flies | Caught | Rate |
|---|---|---|---|
| LA 25–35° | 428 | 269 | **.629** |
| LA 35–50° | 1,299 | 1,132 | .871 |
| LA 50°+ | 206 | 191 | .927 |
| All | 1,933 | 1,592 | **.824** |

(All flies in the park off the wall: 3,758, of which .213 fall.) Real: about .95–.99. The low
flies are the worst, which is the reverse of what the owner sees as natural (a high ball should
be the harder one).

**The read's error on routine flies, for the man nearest the landing point** (his latest belief
of where it comes down against where it does; along the line home → landing, + = believed
deeper; m, mean / sd), and the same read *with every noise draw at zero* from where he actually
was:

| LA | 2.0 s before landing | 1.0 s | 0.5 s | no draws: 2.0 s | 1.0 s | 0.5 s |
|---|---|---|---|---|---|---|
| 25–35° | +3.52 / 4.18 | +1.44 / 3.06 | +0.74 / 1.83 | **+3.22** | +0.90 | +0.23 |
| 35–50° | +1.53 / 3.65 | +0.80 / 2.11 | +0.48 / 1.28 | +1.18 | +0.38 | +0.11 |
| 50°+ | +1.26 / 3.42 | +0.69 / 2.03 | +0.40 / 1.47 | +0.68 | +0.25 | +0.07 |

Lateral errors have mean about 0 and sd 2.4–2.8 m at 2 s, 1.1–1.3 at 1 s, 0.6–0.9 at 0.5 s.

So there are two faults, not one:

1. **A bias: every fly is read too deep.** With no noise at all the read puts a 25–35° fly
   3.2 m deeper than it lands 2 s out. Cause, found by refitting the belief table
   (`examples/batted_spin.rs`, 48 games): the backspin fielders believe a fly has
   (`typical_batted_spin_rpm`, fitted 2026-11-12 on the v1 swing) is 60% higher than what the
   contact model now gives. Medians by launch angle, rpm, belief → current: 25° 3,290 → 1,916;
   35° 4,800 → 3,013; 45° 5,990 → 3,819; 55° 6,470 → 4,558. Swing v2 (2026-11-14) changed the
   spin and the table was not refit; §5.1 says it must be. More believed backspin is more lift:
   the ball is read to carry. Two smaller parts of the same belief push the same way: the
   launch angle the table is looked up at is estimated as atan2(v_z + g·s, v_h) from the ball's
   velocity *now*, which drag has made too steep by mid-flight (a steeper launch angle looks up
   more spin), and the believed spin is the launch spin, though the ball's spin has decayed
   since (`AeroParams::tau100`, 22 s at 100 mph).
2. **Noise that does not close in time.** The spread (sd) is 3.4–4.2 m at 2 s and still 2.0–3.1
   m at 1 s, against a planning reach of 0.7 m. The read's position term is σ₀·(d/30 m) ×3 in depth
   /√(1 + view/τ_read): it shrinks with viewing time and his distance to the ball now, but not
   with how near the ball is to coming down, so a 2σ depth draw is still 4 m off a second
   before landing. 239 of the 341 misses were read too deep at 1 s, 102 too short.

**The misses, by cause** (the responsible man's own log at landing):

| Cause | Misses |
|---|---|
| Got within 1.4–2.5 m of the ball's track (beyond reach and stretch) and did not dive | 144 |
| Read error late: more than 3 m at 1 s or 2 m at 0.5 s before landing | 128 |
| Read within those bounds late, still too far to make up | 35 |
| Reached it (within reach + stretch), not caught: the glove (A17's territory, not this job) | 31 |
| Left it to a teammate who then missed | 2 |
| Nobody chasing at landing | 1 |

- **Not the replan clock.** At landing the responsible man's last read was a median 0.03 s old
  (mean 0.07 s): in the last seconds the close reads (every 0.1 s within 4 m) and the dive reads
  keep him current. `REPLAN_S` 0.4 s matters earlier, in how a frozen belief steers him.
- **Not standing.** 340 of 341 were moving at landing: he is still correcting, not waiting on a
  wrong spot.
- **Not the call.** 3 of 341.
- **"Dive range, no dive" is the read again.** These men were 1.4–2.5 m from the ball's track
  with a read that was still off by 2–5 m at 1 s (e.g. seed 0 play 14, seed 4 play 78, seed 32
  play 27: +4.3, +4.6, +3.9 m deep at 1 s). His read told him he would get there on his feet
  until too late to lay out (the dive needs `DIVE_LUNGE_S` of warning, §5.3). The dive rule is
  not changed here.

**Traced examples** (`trace_play <seed> <play>`; err = read error at 2.0 / 1.0 / 0.5 s before
landing, + deeper):

| Seed, play | Fielder, LA, hang | Need / coverage | Read error, m | What happened |
|---|---|---|---|---|
| 23080 / 8 | LF, 45.6°, 6.2 s | 49 / 129 ft | +10.5, +6.2, +4.0 | Starts the right way, drifts 11 m to the line side and 6 m deep, comes back too late (gap 5.1 m) |
| 0 / 109 | CF, 30.0°, 4.8 s | 48 / 94 ft | +12.9, +9.9, +5.9 | Runs past it; 7.4 m short at landing |
| 9 / 52 | CF, 32.7°, 5.2 s | 30 / 102 ft | −3.5, −2.1, −0.6 | Read short; comes in, gets within 1.6 m running back; no dive |
| 228 / 203 | CF, 37.5°, 5.7 s | 63 / 120 ft | +3.2, +1.5, +0.6 | Read good late, glove within 0.4 m, not caught (glove: not this job) |

Fielder communication's three known misses (seed 11 play 1, seed 4 play 280, seed 2 play 140)
are not routine by this measure (none is in the list).

**Ranked conclusion.** (1) Refit the typical-spin belief to the current contact model and make
it a better belief (launch angle from the ball flown back to the plate; the spin decayed since);
that removes a bias every fielder and runner shares. (2) Make the read's uncertainty close as
the ball comes down, as the research says real fielders' does, and give it a per-fielder rating
so grade 50 lands in the owner's 90–99% band. The dive rule, the replan clock and the call are
not where the misses come from.

## 2. The change (agent-models §5.1, §5.2, §10)

Built in `live.rs` (`believed_spin`, `fly_back`, `read_shrink`, `read_prediction_k`,
`Sim::read`) and `play.rs` (`FielderAttrs::read_vel_frac`, `with_read`, `READ_RHO`,
`TAU_READ_RHO`). Specs: agent-models §5.1 ("The typical spin refit", "The read closes"), §5.2
(replans), §10 (`ReadSkill`).

1. **The typical-spin belief follows the contact model again.** The table is refit to swing v2
   (medians by 10° bin, `examples/batted_spin.rs`, 48 games). The launch angle it is looked up
   at is now the angle at which his velocity read, flown back to the plate with his own physics,
   leaves the bat (not atan2(v_z + g·s, v_h) of the velocity now), and the spin he believes
   the ball has now is that launch spin decayed as the same back-flight undoes it. With no
   noise the read 2 s before landing now misses by −0.1 to +0.3 m (it was +0.7 to +3.2 m).
   This is a belief about the contact model, not a tuning: §5.1 says the table is refit when
   the contact model changes. Runners' reads and the fielders' shared view of a loose ball
   (`public_path`) use the same belief and move with it.
2. **The read closes as the ball comes down.** For a ball on its first flight, every term of the
   read error is scaled by c = min(1, T/`READ_CLOSE_S`), T the time it takes to fall from its
   height now with its vertical velocity now, in a vacuum (what he sees now; nothing of its
   future). *Mechanism and source:* fielders are continuously coupled to the ball and the
   information about where it lands sharpens late (optical acceleration cancellation: Chapman
   1968; Fink, Foo & Warren 2009, J. Vision 9(13):14, research note §1). With a threshold on the
   optical acceleration he can perceive, the landing miss he can tell apart shrinks with the
   square of the time left; the velocity term, flown over T and scaled by T, does that.
   `READ_CLOSE_S` = 3.0 s [E] is where the closing starts. It is fitted to the owner's band; no
   source gives it. A high fly keeps its larger early error (1 s after contact a 45° fly's
   landing spread is 5.5 m against 4.9 m for a 28° fly, test
   `the_read_closes_as_the_ball_comes_down`) and the first steps still follow the first read.
3. **Faster replans late.** A chaser reads again after a quarter of the time his own read leaves
   before the ball comes down, between 0.1 and 0.4 s (`REPLAN_FRAC`, `REPLAN_MIN_S`), so a read
   that sharpens late is used late. (The spec's 30 Hz is still not built; the diagnosis showed
   the stale replan was not a cause of misses on its own.)
4. **`ReadSkill`, a per-fielder rating.** `read_sigma0`, `read_vel_frac` and `tau_read` are
   per-fielder values (the grade-50 values are the old constants); `FielderAttrs::with_read(g)`
   scales all three by ρ = 1.06 per 10 grades; `sim-cli games --read <g>` gives every fielder
   grade g. Every rostered player is still grade 50.

**What was tried and rejected.**
- *A read that never gets worse ("what he has seen he does not unsee").* Seed 23080 play 8's
  left fielder reads the ball well at first (4.6 m off) and *worse* at the top of the flight
  (13 m off at 2.4 s before landing; `examples/read_track.rs 23080 8 7`): the plate cue fades as
  the spin he cannot see adds up over the back-flight, and his direct read's depth error is that
  of a ball 40 m away. Two forms: capping his position error at the best σ he has had made
  grade 50 catch .989 of routine flies with no closing at all and grade 20 .970 (the early
  plate-cue σ held for the whole flight, an observer who knows where the ball is to 0.3 m at
  3 s); carrying that best read forward as the plate cue carries the contact point (velocity
  error × elapsed time, plus `PLATE_SPIN_M` × Δ^1.5) barely bound (the carried σ grows as fast as
  the direct read's). Both removed. The mid-flight degradation is still in the model (§4); the
  closing now starts near the top of a high fly (T at the apex of a 40 m fly is 2.9 s in a
  vacuum) and pulls the read back in time. A proper fix is a filter on the whole observed
  track, which would learn the ball's curvature (its spin) as it flies. Not built; §5 Q2.
- *`READ_CLOSE_S` 1.5–2.5 s.* 2.5 s missed 23080/8; over seeds 0–299 it gives grade 50 .949 of
  routine flies, fly-ball hits .109, runs 3.42 (seed set 0 only). 3.5 s gave .976 on 120 games.
- *ρ 1.4 or 1.1 for `ReadSkill`.* 1.4 put grade 20 at .536 (the catch rate is steep in the read
  error); 1.1 put it at .85–.86. The old §10 ρ 1.2 on τ_read alone held grade 20 near .87.
- *Re-fitting `PLATE_SPIN_M` with the new belief.* Re-measured (`examples/plate_cue.rs`, 48
  games): 0.6 m at 0.5 s, 2.0 m at 1 s, 4–8 m at 2 s, as before. The plate cue's floor is the
  spread of spins and their side component, which the median does not remove. Unchanged.
- *The dive rule, the call, the replan clock alone.* Not where the misses came from (§1).

## 3. Measurements (main `dc1ce82` ≡ `50ed674` → `a94a7a2`)

**Standard run**, `sim-cli games --n 300`, seeds 0–299 [`--seed 1000`, game seeds 1,000,003,000 on]:

| | Before | After | Real |
|---|---|---|---|
| Runs / team-game | 4.42 [4.59] | **3.32 [3.42]** | 4.39 |
| PA / team-game | 39.0 [39.2] | 37.1 [37.0] | |
| K% / BB% | 21.6 / 8.1 [21.9 / 8.4] | 21.5 / 8.2 [22.0 / 8.4] | |
| HR / PA | 1.87% [1.91%] | 1.88% [1.87%] | |
| Reached on BIP | 39.1% [39.1%] | 32.4% [33.0%] | |
| GB hits | .322 [.319] | .293 [.304] | ≈ .24 |
| LD hits | .762 [.774] | .653 [.659] | ≈ .68 |
| **FB hits** | .219 [.228] | **.099 [.102]** | ≈ .13 |
| PU hits | .030 [.033] | .011 [.011] | ≈ .02 |

**Routine flies** (`fly_misses 0 300`, §1's definition), grade 50: **.824 → .967** (25–35°
.629 → .950; 35–50° .871 → .972; 50°+ .927 → .973). Every fly in the park off the wall: .213 →
.083 fall. By `ReadSkill` (every fielder at the grade, seeds 0–119): grade 20 .917, grade 50
.960, grade 80 .983. Seed 23080 play 8 is caught (glove at 6.54 s; it would land at 6.55 s).
The read's error on routine flies at 2.0 / 1.0 / 0.5 s before landing (sd in depth): 2.2–2.9 /
0.7–0.9 / 0.2–0.4 m (was 3.4–4.2 / 2.0–3.1 / 1.3–1.8); with no draws the bias is now under
0.35 m at 2 s.

**The 60 routine misses left** (of 1,799): 31 within 1.4–2.5 m of the ball without a dive; 19
read well late but too far to make up; 5 reached and not held (the glove, A17's); 3 left to a
teammate who missed; 2 read errors late. None of the causes is now dominant.

**Mechanism of the fall in runs.** Every fly that is now caught is an out that used to be a
hit: fly-ball hits fall by .12 of 22.5% of balls in play and pop-up hits by .02; reached on BIP
falls 6.7 points. Grounders (.322 → .293) and liners (.762 → .653) fall too, from the same two
changes: the typical-spin table's topspin side moved as much as its backspin side (−7,060 →
−4,856 rpm at −25°), so infielders' and outfielders' reads of grounders and liners lost a bias
as well, and a liner's whole flight is inside `READ_CLOSE_S`. Nothing was compensated anywhere.
The paired fly replay was not run.

## 4. What is still off

1. **Flies now fall too rarely: .099 [.102] against a real .13**, and pop-ups .011 against
   .02. Lowering `READ_CLOSE_S` to 2.5 s gives .109 and leaves grade 50's routine rate .949, so
   the read is not the main lever. The research note's estimate (§3 there) is that the sim's
   average outfielder already covers in 3 s what an elite real one does (35 ft against
   29–38 ft for real season means); with the read fixed that may now show. Not measured here.
2. **The read still degrades in mid-flight on high flies.** On 23080/8 the left fielder's read
   goes from 4.6 m off at his first look to 12.9 m off at the top (2.4 s before it lands) and
   comes back only as the closing takes hold; he drifts 9 m deep and gets back just in time.
   The cause is in the model, not in the draw: the plate cue fades with the unseen spin
   (`PLATE_SPIN_M`·s^1.5) while the direct read's depth error is that of a ball 40 m away. A
   real fielder keeps all he has seen; a filter over the observed track would (Q2).
3. **High flies are not harder than low ones on routine plays** (.973 against .950 at 25–35°),
   which is the reverse of the owner's "a high ball is harder to judge". Early in the flight the
   high fly's read is the looser one (test), but routine high flies also give the fielder
   longer, and the 25–35° flies include balls with less hang time. Not pursued.
4. **Runs 3.32 [3.42] against a real 4.39.** They sat near real because errors offset each
   other; this removes one. Grounders are still too often hits (.293 against .24).
5. The read's rating (`READ_RHO` 1.06) is fitted with every fielder at one grade; with a
   population of ratings it should be refit.

## 5. Questions for the owner

**Q1. Fly balls now fall .10 of the time against .13 in MLB. What happens now:** the
outfielder's read is fixed by your target (a routine fly caught 90–99% by ability; grade 50
.967) and runs fell to 3.32 [3.42]. **Problem:** the last two points of fly-ball hits are not the
read (a looser read, 2.5 s, still gives .109). **Options:** (a) leave it and diagnose the
outfielders' speed and first step against Statcast's feet covered in 3 s next (the research
note's P7, which it rated unlikely before the read was fixed); (b) loosen the read until flies
fall .13 (grade 50's routine rate would drop below .95); (c) tune runs back elsewhere.
**Recommendation: (a).** The owner's band for the read is met; (b) would put error back into
the one mechanism just measured against it; (c) is against the calibration rules (no
compensating knobs).

**Q2. A fielder's read still gets worse in mid-flight on high flies (§4.2). What happens now:**
the closing read pulls it back late, so the catch is made, but the man drifts the wrong way at
the top of the flight. **Options:** (a) a filter over the whole observed track (a Kalman
filter on the ball's state, with the unseen spin as a slowly learned acceleration), replacing
the plate cue and the frozen-draw read: the principled fix, a few days with tests and the
isolation harness; (b) leave it, since the catch rate is in band. **Recommendation: (a), as its
own job**, after Q1's diagnosis, since what the owner sees (drift) is the read, not the rate.

## Added after the Codex review (lead, 2026-11-15)

The review ([docs/reviews/2026-11-15-codex-outfield-read-review.md](../reviews/2026-11-15-codex-outfield-read-review.md)) found no P1 and calls this an interim read model. Two statements above are withdrawn or qualified by it. (1) "The last two points are not the read" and the suggestion that the outfielders' legs are elite: not established. Its probe measures the nearest outfielder covering 33.6 ft in the first 3 s on plays needing at least 60 ft (MLB's worked example implies a 34.8 ft baseline) with a 26.9 ft/s cap (MLB average sprint speed 27 ft/s); the cause of flies falling .10 against .13 is unresolved between the read's late precision and movement. (2) The deep drift on high flies (23080/8: 12.9 m at 3.59 s) is the read keeping no track of what it has seen; fixing it is a separate job (assimilating successive observations). The spin belief must be refit when the contact model changes (`work/collision-q`).
