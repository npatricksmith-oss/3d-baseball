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
