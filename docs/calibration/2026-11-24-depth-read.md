# The depth read of a ball seen on the ground: seed 65147 play 10 (2026-11-24)

Branch `work/depth-read`, from main `aa4a31d`. Spec: [agent-models §5.1](../agent-models.md#51-perception-after-contact), *The depth of a ball seen on the ground*. Probe: `sim/crates/bb_game/examples/depth_read_probe.rs` (`-- <seed set> <games>` for the counts below; `-- play <seed> <play> <fielder>` for one fielder's reads).

## 1. What was wrong

A fielder whose first look comes after the ball's first bounce has no track (the [bounce-track note](2026-11-21-bounce-track.md) §3 dropped a fresh track there). He reads the ball by the snapshot read: its depth σ along his line of sight is `DEPTH_STRETCH` × σ0·d/30 m, 28 m at 80 m for an outfielder, shrunk only by viewing time (1/√(1 + view/τ_read)), and the play's fixed depth draw (truncated at 4σ) is applied at it on every replan. The outfielders' reaction is 0.35 s, so this is every ground ball that bounces before then, for all three of them.

On the frozen fixture of seed 65147 play 10 (`tests/fixtures/a4/65147-10.json`) the right fielder's read after the bounce was up to 32 m off along his line of sight (37 m in the saved game). On seed 1 play 66 the centre fielder, 92 m away, read a chopper to the mound as 107 m beyond it, behind him, and turned for the wall.

## 2. How often, and what it cost (main, 300 games at each seed set)

- **How often.** 22.2 [21.3] plays a game have outfielders with no track at the first bounce: 66.5 [63.9] outfielder-plays a game, of which 25.0 [24.0] chase the ball at some point (the chase role).
- **How far off** (his read of where the ball is now, along his level line of sight, while chasing): median 15.5 m 0.5 s after the bounce, 9.9 m at 1 s, 4.6 m at 2 s; 90th percentile 43.6 m at 0.5 s. Chasers ever more than 20 m off: 11.8 [11.2] a game; more than 10 m, 17.6 [16.7].
- **What it cost, at most.** A throwaway build with no depth error at all for those outfielders (both depth draws zero on the batted ball after its first bounce, before any wall) is the upper bound of what any better depth read can do: runs 4.09 → 4.15 [4.37 → 4.34] (opposite signs, within the sets' spread), ground-ball hits .307 → .309 [.312 → .313], ground-ball triples 108 → 96 [93 → 83] and doubles 362 → 357 [371 → 360] in 300 games, an outfielder securing a ground-ball hit 3.92 → 3.86 s [3.89 → 3.84 s] after contact.

So the cost in outcomes is small, as the owner's rule says a small effect is not worth building for. It was built anyway because the behaviour is not one a real player has (AGENTS.md: "Judge a behaviour change by 'would a real player do this?', not only by its run cost") and it shows about twelve times a game: an outfielder breaking for a spot tens of metres from a ball rolling on the grass in front of him. The change is small and touches one read.

## 3. The model (agent-models §5.1)

A ball on the ground at level range d is seen at an angle δ below his eye level, tan δ = h/d (h his eye height less the ball's radius, 1.66 m). Distance on the ground is perceived through this angle (Ooi, Wu & He 2001, *Nature* 414:197). An error σ_δ in it moves the range by (d² + h²)/h · σ_δ. σ_δ = `GROUND_DECL_SIGMA_RAD` = 3′ at grade 50 [E]: within 15% of the same eye's `Vision` σ_ang in the batter's read (1.0 mrad = 3.4′ at grade 50), here placing the ball against the field around it; scaled by `ReadSkill` (`READ_RHO`) as the other read σs (`FielderAttrs::ground_decl_sigma`). It gives 0.5 m at 30 m, 1.9 m at 60 m, 3.4 m at 80 m, 5.3 m at 100 m.

He has the cue while the ball rolls, on a hop from where it last touched the ground (at that point's range, σ grown by his velocity read along his line of sight times the time since), and, since the review, on a new ball that starts within 0.3 m of the ground (off a glove's rim, a wall, a throw come down) from where it started, with the height it may have had there as a doubt (§8). It is weighed against the snapshot read's depth σ by precision, and the play's fixed depth draw is applied at the result. Nothing else in the read changes; no new draws.

*Why not a track started at the bounce (dropped 2026-11-21).* That track diverged on far balls because its persistent error was a fraction of a range it believed short, and its fresh starts made the infielders' first half-second rougher (ground-ball hits .308 → .334). Here nothing carries a persistent error scaled by a believed range: the snapshot read applies its draw at the σ worked out at the true range (the world's draw, as before). The tracking read is untouched.

*Why not the range rate.* Successive looks at the angle also give how fast the range changes, but over T seconds of looks every 0.1 s its σ is about σ_g·√(1.2/T³): 3.7 m/s at 80 m after 1 s, against 1.8 m/s for his velocity read along his line of sight on a 30 m/s ball. It is worth using only after about 2 s, when the ball is near him.

*Who else it changes.* Every untracked read of a ball on the ground goes through it, so besides the outfielders: infielders who first look after a hard grounder's bounce (about 0.30 m at 25 m, from 0.75 m), the pitcher and the catcher the same way, idle fielders choosing a cover or backup job (the centre fielder of seed 1 play 66), and every fielder's read of a ball on the ground after a wall carom, a glove or a throw come down (those end every track). A first look on a grounder still in the air, and every tracked read, are unchanged.

## 4. The two plays, before and after

- **Seed 65147 play 10** (frozen fixture): the right fielder's worst read after the bounce, along his line of sight, 31.9 m → 4.2 m (4.8 m after the review's fixes, §8). He still fields it (`seed_65147_play_10_an_outfielder_takes_a_grounder_through_the_hole_both_left_to_the_other` passes).
- **Seed 1 play 66**: the centre fielder's depth draw is about −3.4σ. Before, his read put the chopper 107 m off, behind him, and he turned for the wall; now it is 15 m off (4.4 m σ at 92 m), and he takes a backup job toward the left side.

## 5. Measurements (standard run; the ledger row has them all)

First build (before the review), main → branch, seeds 0 [1000]: runs 4.09 → 4.10 [4.37 → 4.32]; GB hits .307 → .309 [.312 → .308]; LD .670 → .666 [.662 → .659]; FB .098 → .102 [.101 → .102]; HR/PA 2.75% → 2.79% [2.84% → 2.82%]; reached on BIP 33.7% → 33.7% [33.8% → 33.7%]. All within the sets' spread. The numbers after the review's fixes are in §8.

From the probe, 300 games: chasers ever more than 20 m off in depth 11.8 → 0.42 [11.2 → 0.39] a game (the perfect-depth bound: 0.38 [0.39]), more than 10 m 17.6 → 1.24 [16.7 → 1.23]; median depth error while chasing 0.5 s after the bounce 15.5 → 2.1 m, at 1 s 9.9 → 1.4 m; ground-ball triples 108 → 89 [93 → 89], doubles 362 → 355 [371 → 334]; an outfielder secures a ground-ball hit 3.92 → 3.84 s [3.89 → 3.85 s] after contact. The extra bases are within noise and beyond the outfield-only perfect-depth bound of §2, so they are not attributable to the outfielders' faster pickups (§8).

## 6. Tests, and the mutations each fails under

- `the_range_of_a_ball_on_the_ground_follows_its_angle_below_his_eyes` (live.rs): `ground_range_sigma` against the geometry itself (h / tan(atan2(h, d) ± σ)), 2–110 m, to 0.1%. Fails with d²/h for (d² + h²)/h (at 2 m, 2.4e-6 against 4.1e-6).
- `a_ball_on_the_ground_is_read_where_his_eye_puts_it` (live.rs): a grounder up the middle, read by the snapshot read by the shortstop and the centre fielder with their depth draw at ±1σ and ±2σ: (1) the ground cue is absent before the first bounce, the time since the last bounce on a hop and 0 rolling, against the world's path; (2) while it rolls, the angle below his eyes of where he reads it is within draw × σ_δ (15% for the geometry's curvature at 2σ) of the true one. Fails with no cue (the shortstop at 30 m, 9.2′ against 6.9′), with the cue's σ doubled (7.4′), and with no cue on hops (part 1).
- `seed_65147_play_10_the_right_fielder_reads_the_chopper_on_the_grass_within_a_few_metres` (tests/throw_choices.rs): old against new on the frozen play, the right fielder's worst read after the bounce under 8 m (4.2 m; 31.9 m with no cue, which fails it).

`cargo test --workspace`: 578 passed, 0 failed, after a touch.

## 7. Open

- The cue as a measurement of the tracking read's looks (a tracked fielder after the bounce is already a median 1.6 m off at worst in 3 s, bounce-track note §10, so little to gain), and its range rate once the ball is near.
- The doubt an untracked fielder's call weighs (`landing_depth_sigma`, a catch in the air only), the runners' reads of a ball on the ground and the receiver's read of a throw on the hop do not use it.
- σ_δ 3′ is [E], anchored to `Vision` σ_ang; the cue itself is sourced. A grade-20 fielder has 4.6′.
- His eye height bobs while he runs: an error σ_h moves the range by d·σ_h/h, 1.8–3% of it for 3–5 cm, as large as the angle's term within about 60 m. Not modelled, so the cue is a little too sharp for infielders.
- A ball off the wall in the air (against a known plane) has no cue until it next touches the ground.
- A persistent draw near its 4σ truncation still misplaces a far ball by about 15 m (seed 1 play 66), as the snapshot read of a fly would; such draws are rare (one fielder-play in about 1,500).

## 8. After the Opus review (2026-11-24)

[Review](../reviews/2026-11-24-opus-depth-read-review.md): one P2, six P3. Each was checked before fixing.

- **P2-1, a grounder off a glove's rim lost the cue for its first hop.** Confirmed in the code: the rim pops the new ball up at 1 m/s from at least 1 cm above the ground, it had not bounced, so no cue, and the new ball's call round re-reads every fielder at once. Fixed: a new ball (not the one off the bat) that starts within `GROUND_START_Z_M` = 0.3 m [E] of the ground and has not touched the ground or a wall yet is seen on the ground where it started; the height z it may have had there is a doubt of d·z/h in its range, in quadrature (`Sim::ground_seen`, `GroundCue`). Test: `a_grounder_off_a_gloves_rim_is_read_on_the_ground_where_it_left_the_glove` (three grounders knocked off a rim at about 25 m, each outfielder's read right after it is within his draw × hypot(σ_δ, z/d) in angle; 2.9–6.5′ against bounds of 3.5–7.0′). It fails when the new low ball gets no cue. Two ground-dive tests whose fixtures loose a ball 4 cm up (`loose_ball`) had been tuned to the centre fielder's read of it with his draws, metres off in depth: now that read is exact. Their fixtures now take no read error and were retuned to the windows their comments state (10.5 → 10.3 m/s, 2.6 → 3.5 m off); their assertions are unchanged.
- **P3-1, the hop's σ at the range now.** Confirmed: the angle he measured was at the bounce point. Fixed: the declination term is taken at the bounce point's range (`a_hop_is_read_from_the_angle_of_its_bounce`, a hop from 43 m now at 30 m: the read's depth offset, put back at the bounce, moves the angle by 0.85–1.15 of his draw × σ_δ; it fails with the range now). *The run-on term stays independent of his velocity draw `err[2]`, decided:* honestly it is his own speed misread, but the snapshot read carries his whole depth misjudgment as one persistent draw at one σ, and tying a part of it to another draw would split that draw by the weighing; it is a few tenths of a metre on a hop (about 0.4 m for 0.3 s at 20 m/s), against the cue's metres at outfield range.
- **P3-2, no test failed when the cue was too sharp.** Confirmed. `a_ball_on_the_ground_is_read_where_his_eye_puts_it` is now two-sided where the centre fielder's snapshot depth σ is at least five times the cue's (then the weighing gives at least 0.98 of the cue's, so his angle is off by at least 0.85 × draw × σ_δ; 360 draws held to it). It fails with the cue ten times sharper. The frozen-play test's comment now says it bounds that play only while its course holds.
- **P3-3, who else changes.** Agreed; named in the spec, §3 and the ledger row.
- **P3-4, the extra bases.** Agreed; reworded: within noise and beyond the outfield-only bound, not attributable to the outfielders' pickups.
- **P3-5, the 3′.** Agreed; anchored to `Vision` σ_ang, kept [E], the eye-height term listed as open (§7).
- **P3-6, nits.** The observations line no longer names the branch; the spec's crossover reads "anywhere on a field for an outfielder, within about 57 m for an infielder".

Not caught by a test: dropping the height doubt (z/d) itself. On a rim deflection the ball starts 1 cm up, so the term is negligible there, and a lower bound needs the cue to dominate the snapshot read, which it does not once the ball starts high.

**After the fixes** (main → branch, seeds 0 [1000]; jobs 263, 264): runs 4.09 → 4.10 [4.37 → 4.35]; GB/LD/FB/PU hits .307/.670/.098/.005 → .306/.668/.101/.004 [.312/.662/.101/.007 → .308/.660/.101/.007]; HR/PA 2.75% → 2.77% [2.84% → 2.83%]; reached on BIP 33.7% → 33.7% [33.8% → 33.6%]: within the sets' spread, and ground-ball hits did not rise. Probe: chasers more than 20 m off in depth 11.8 → 0.16 [11.2 → 0.13] a game, below the outfield-only perfect-depth bound (0.38 [0.39]), which left a ball knocked off a glove as it was; median depth error while chasing 0.5 s after the bounce 15.5 → 2.4 m; ground-ball doubles 362 → 358 [371 → 334], triples 108 → 91 [93 → 91] (doubles and triples over both sets 934 → 874, the bound 896; within noise). On the frozen seed 65147 play 10 the right fielder's worst read after the bounce is 4.8 m (31.9 m on main; 4.2 m in the first build, whose hop σ was taken at the nearer range now). `cargo test --workspace`: 580 passed, 0 failed.
