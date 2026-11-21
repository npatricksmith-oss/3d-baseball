# A fielder keeps tracking the ball across its bounces: the owner's saved play 71 of seed 51001 (2026-11-21)

**Job.** The fix the [saved-plays diagnosis](2026-11-20-saved-plays-51001.md) §2 proposed as Q-B (a), chosen by the Coordinator as a fix that brings the read in line with what a real fielder does: a bounce continues the fielder's tracking read instead of ending it, and no read puts the ball below the ground. Branch `work/bounce-track`, from main `519842c`. Spec: agent-models §5.1, *Across the bounces* and *No read puts the ball below the ground*.

## 1. What was wrong

Each fielder's tracking read (agent-models §5.1) ended at the ball's first bounce, touch or wall. After it every read was a fresh snapshot: the ball's true state plus his whole fixed error draw for the play, three times larger along his line of sight (`DEPTH_STRETCH`), on every replan. In play 71 the right fielder's read of a liner was 4–5 m short in flight (his fixed draws are 3.1σ in range and 3.4σ in speed: a big but believable misjudgment). At the bounce, which he watched from 26 m, his read jumped to 15 m off, and the height noise put the ball 0.24 m under the ground going up, on a believed path that never came down. He turned for the wall at 2.3 s; the ball passed 5.7 m from him and the batter had a triple.

## 2. The model (agent-models §5.1)

- **A seen bounce continues the track.** At the moment he sees the ball touch the ground, (1) its height there is the ground's, a measurement that also corrects the range he misjudged together with the height; (2) it bounces as he believes a bounce goes: his own bounce law, on the surface under his estimate, with the spin he believes it meets the ground with (the first flight's typical spin; after that, what his law left it with at the bounce before), no tilt. The covariance goes through the law's Jacobian, plus what he cannot foresee (the surface's roughness tilt, and 1,000 rpm of spin he cannot see); (3) the learned extra acceleration returns to its prior; (4) his persistent error carries over. Nothing is drawn.
- **A hop** flies on with the spin his law left it with. **A rolling ball** has its height and vertical speed known, and rolls on by the world's rolling law as set where it began to roll (μ_r·g of that surface, or the even slowing that stops it at the wall); what that law has wrong he learns as an extra level acceleration. **A ball at rest** has its velocity known.
- **Off the first flight** a velocity look is no better than for a ball going 3 m/s, and a look his estimate cannot explain at all (normalised innovation squared above 50) ends the track. After that, and after a wall carom, a touch, or a new loose ball, he reads the ball by the snapshot read, as before.
- **A fielder with no track at the first bounce** (his reaction ended after it) reads by the snapshot read, as before.
- **No read below the ground**: a snapshot read of a ball seen rolling takes its height and vertical speed as known; every other read is held at the ball's radius at least. This applies to the runners' reads too.

**Parameters** (all [E], `live.rs`):

| Name | Value | Source or reasoning |
|---|---|---|
| `BOUNCE_Z_M` | 0.01 m | Numerical: the height at a seen touch is the ground's; the σ only keeps the update well conditioned |
| `BOUNCE_SPIN_RPM` | 1,000 rpm | Unsourced: about half a liner's typical backspin; checked by the honesty measurement (§5) |
| `ROLL_SLOW_PRIOR` | 0.6 m/s² | Unsourced: grass's μ_r less dirt's, times g |
| `ROLL_DRIFT` | (0.6/2)² / 2 m²/s⁵ | Half the prior's change over a 2 s roll |
| `LOOK_VEL_FLOOR_MPS` | 3 m/s | Unsourced: a speed read is a fraction of the speed only while the ball moves well |
| `TRACK_GATE` | 50 | χ² with 3 degrees of freedom passes 50 once in 10¹⁰ looks of an honest track |

The world's roll code moved, unchanged, into `bb_physics::ground::roll_from`, so the fielder's belief and the world use one rolling law.

**Draws.** The looks are the only draws: the existing `FielderLook` stream, its sub-index now the look's number in the play (it counts on past the first bounce). No new purpose; every look on a first flight draws exactly as before.

## 3. What was built first, and dropped

1. **A fresh track for a ball first seen after it bounced** (and for a new ball off a wall or a glove), started from the look against a wide prior with the persistent error carried. On far balls the persistent error is a fraction of the range, and an estimate started tens of metres short believes that fraction of a *short* range, so it grew sure of a wrong range (a centre fielder 40 m off with σ 1.6 m). The infielders' fresh tracks of hard grounders were honest but rougher than the snapshot read for their first half-second, and ground-ball hits rose from .308 to .334 (runs 4.19 → 4.28 on seeds 0–299). Dropping the fresh tracks put ground-ball hits back (.312 on games 0–95, as main).
2. **The roll slowing at the surface under his estimate, with a velocity random walk** (`ROLL_DRIFT` as velocity noise). The world keeps a roll's first surface and stops a roll that would reach the wall by an even slowing from its start; a ball that rolled from the dirt onto the grass, or into the wall, slowed other than he believed, and his error²/σ² 2 s after the bounce was 13 for outfielders. Now the rolling law is set as the world sets it, at the roll start, and the rest is learned.
3. **600 rpm of topspin after every bounce** (the flight model's belief after a bounce). A hopping grounder carries thousands of rpm of topspin from its bounces, so his predicted hops were badly wrong: in games 0–9 the gate ended 122 tracks, 115 of them on hops. With the spin his own bounce law gives carried through, it ended 9, all on rolling balls.
4. **The velocity look's σ in proportion to the ball's speed down to zero.** As a ball came to rest its velocity looks became exact and the filter's gain ran away (one estimate moved 30 m in one look). Hence the floor, and a ball seen at rest has its velocity known.

## 4. Play 71, before and after

The ball and the right fielder's draws are the same (the play's own streams). Probe: his read along his line of sight, from his track (`read_trace`, now with the estimate and its σ).

| t (s) | Before: read | After: read error along his line of sight | After: his σ there |
|---|---|---|---|
| 2.12 (before the bounce) | 4–5 m short | +4.1 m | 1.5 m |
| 2.24 (just after) | **15 m off, z −0.24 m**, snapshot | +2.5 m | 1.4 m |
| 2.6 | snapshot, 15 m class | +2.9 m | 1.5 m |
| 3.0 | snapshot | +1.8 m | 1.0 m |
| 3.3 | snapshot | +0.1 m | 0.3 m |

Before, he turned for the wall at 2.3 s and the ball passed 5.7 m from him. Now he keeps running at the ball's line, and his read is right to 0.1 m as it reaches him; the ball passes 1.9 m from him at 3.4 s, rolling at about 16 m/s. He is a step short, does not dive at it, and it rolls to the wall: still a triple. What is left is the in-flight misjudgment (he ran 4 m short and to the wrong side while the liner was up). With a true read from the bounce on (a probe counterfactual), he is 1.4 m from the ball at 3.4 s, dives, knocks it down and holds the batter to a single (secured 6.45 s at (35.2, 78.1), the diagnosis's "about 6.4 s"). So the read after the bounce is now no longer the fault; the play is decided by a believable 3σ misjudgment in flight and by whether he dives at a ground ball 1.9 m away (outside this job).

The fixture test `the_owners_rolling_ball_seed_51001_play_71_is_played_at_its_line` keeps the ball (its contact, its streams): his read a moment after the bounce no worse than just before it (2.48 m against 4.17 m), within a metre by 3.3 s (0.08 m), and the ball passing within 2.5 m of him.

## 5. Is the read honest, and not too good?

`zz_bounce_honesty` (probe, kept in `sim/target/codex-probes/bounce/`): every fielder who was tracking the ball before its first bounce, games 0–59, his estimate's error along his level line of sight against where the ball truly is, and his own σ there. The in-flight row (−0.05 s) is the same balls on their first flight.

| From the first bounce | Outfielders: RMS error, RMS σ, mean e²/σ² | Within 40 m of the ball | Infielders | Within 40 m |
|---|---|---|---|---|
| −0.05 s (in flight) | 1.44 m, 1.09 m, 1.42 | 1.06, 0.88, 1.33 | 0.22, 0.22, 0.96 | 0.17, 0.17, 0.88 |
| +0.05 s | 1.24, 0.97, 1.39 | 1.02, 0.85, 1.30 | 0.21, 0.21, 0.96 | 0.17, 0.16, 0.93 |
| +0.5 s | 1.27, 0.99, 1.37 | 0.90, 0.78, 1.09 | 0.22, 0.21, 1.02 | 0.14, 0.14, 0.98 |
| +1 s | 1.34, 1.00, 1.42 | 0.81, 0.71, 1.10 | 0.24, 0.23, 1.03 | 0.12, 0.12, 0.97 |
| +2 s | 1.14, 0.84, 1.37 | 0.51, 0.54, 1.05 | 0.27, 0.24, 1.16 | 0.11, 0.10, 1.19 |

So the read after a bounce is no worse than just before it (outfielders 1.44 → 1.24 m RMS; infielders 0.22 → 0.21 m), it is honest (error²/σ² 0.9–1.4, the in-flight filter's own range; the track-bias note had 0.83–1.36), and it is not too good: the errors are what the in-flight filter has. A fielder running in improves it (within 40 m, 1.06 → 0.51 m over 2 s); one standing still keeps his persistent error, which looks from one place cannot separate from the ball. For comparison, the snapshot read it replaces had, a moment after the bounce, an RMS error of 13.3 m along the line of sight for outfielders standing at their places (test `a_seen_bounce_continues_the_track`, against 1.14 m tracked).

## 6. A handful of plays, before and after

First play that differs in each of games 0–5 (`zz_facts` on main and on the branch):

- **Seed 1 play 44** (81 mph at 8°, a low liner that bounces in front of the shortstop): before, it got through to the centre fielder at 4.9 s and the batter took second; now the shortstop, still tracking it after its hop, takes it at 1.93 s: a single.
- **Seed 4 play 1** (104 mph grounder through the right side to the wall): the right fielder picks it up at the same moment (6.64 s), throws 0.03 s sooner, and the batter, who before ran to second and on to third, now stops at first.
- **Seeds 0, 2, 3, 5** (a chopper to second, grounders to the infield and the outfield): the same plays, the throw released 0.0–0.03 s sooner or the catch a few centimetres away. Nothing a real fielder would not do.
- **Seed 65147 play 10** (the TASKS depth-read item): a chopper that bounces at 0.45 s, before any outfielder has reacted, so none has a track and they read it by the snapshot read as before (the right fielder's first read 37 m off). Not covered (§7).

## 7. Measurements (standard run)

300 games each, `sim-cli games --n 300 --seed 0` [`--seed 1000`]; doubles, triples and the extra bases from `zz_bounce_stats` on game seeds 0–299 (the batter's furthest base on a hit; the extra bases by the official scorer's ruling).

| | main `519842c` | this branch | MLB |
|---|---|---|---|
| runs / team-game | 4.19 [4.59] | **4.09 [4.42]** | 4.39 |
| GB hits | .308 [.321] | .309 [.316] | about .245 |
| LD / FB / PU hits | .672 / .102 / .005 [.673 / .098 / .007] | .671 / .103 / .005 [.664 / .101 / .007] | |
| HR/PA, K%, BB% | 2.73%, 22.5, 8.1 [2.82%, 22.3, 8.5] | 2.74%, 22.6, 8.0 [2.93%, 22.2, 8.4] | |
| reached on BIP | 33.7% [34.5%] | 33.8% [34.0%] | |
| singles / doubles / triples (seeds 0–299) | 2,965 / 1,144 / 521 | 3,090 / 1,140 / **424** | |
| triples a game | 1.74 | **1.41** | about 0.3 |
| ground-ball triples | 214 | **141** | |
| first to third on a single | .343 (350/1,019) | .343 (385/1,124) | .317 |
| second to home on a single | .552 (256/464) | .535 (275/514) | .617 |
| first to home on a double | .795 (232/292) | .786 (213/271) | .38 |
| rules warnings | none | none | |
| ms per game per core | 4,574 [4,726] | 4,474 [4,594] | |

**Causes.** Outfielders who keep their read across the bounce cut balls off sooner: a fifth of the triples (and a third of the ground-ball triples) become singles, which is why singles rise by 125 and doubles stay put. Runs fall 0.10 [0.17]. Ground-ball hits do not move (the infield read across a bounce was already good from the snapshot read at those distances). Second to home on a single falls 1.7 points, within the noise of 500 chances. The direction is the diagnosis's expected one.

**Unexplained:** nothing beyond the noise. Triples remain far above MLB's (1.4 a game): the read is no longer the cause in play 71; the in-flight misjudgment and the ground-ball dive are next.

## 8. Tests

- `no_read_puts_the_ball_below_the_ground`: a rolling ball and a low hop, from 10, 30 and 80 m, every sign of the four draws at 4σ, by the snapshot read and a runner's look: no believed point under the ball's radius, every path meets the ground or leaves the park. Independent: a physical invariant (before, the rolling ball with draws (−4, ·, ·, +4) started under the ground going up and never came down).
- `a_seen_bounce_continues_the_track`: liners and low drives over a grid of directions, the three outfielders at their places, 30 plays of draws each (720 reads): the RMS error a moment after the bounce no larger than just before (1.14 against 1.16 m), no growth a second on in units of a look's σ at that range, error²/σ² within a factor of two of 1 (0.94–1.05), better than the snapshot read at the same moment (13.25 m), and every read meets the ground. Independent: the true ball, the unit-normal property, the old read.
- `the_owners_rolling_ball_seed_51001_play_71_is_played_at_its_line`: the owner's ball (§4).

## 9. Open

- **The depth read of a ball first seen on the ground** (TASKS §2, seed 65147 play 10) is not covered: a fielder with no track at the first bounce keeps the snapshot read. A track started there needs the persistent error's range-proportional size handled other than by linearising at the estimate (it diverges on far balls), and its infield first half-second must not be worse than the snapshot read's.
- **Play 71 is still a triple**: the right fielder is a step short of a ball rolling 1.9 m from him and does not dive at it. Whether an outfielder should dive at a ground ball going by is its own question.
- **A wall carom or a touch** still ends the track (the snapshot read after it); a track that the gate ended likewise.
- **The world's roll keeps its first surface** (physics-spec §9.2 says to switch at the dirt–grass edge; not built, ledger §5). The fielder's belief follows the world.
