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

A ball on the ground at level range d is seen at an angle δ below his eye level, tan δ = h/d (h his eye height less the ball's radius, 1.66 m). Distance on the ground is perceived through this angle (Ooi, Wu & He 2001, *Nature* 414:197). An error σ_δ in it moves the range by (d² + h²)/h · σ_δ. σ_δ = `GROUND_DECL_SIGMA_RAD` = 3′ at grade 50 [E, unsourced as a number]: the ball's own angular size at 80 m, placed against the field around it, three times the eye's 1′ acuity; scaled by `ReadSkill` (`READ_RHO`) as the other read σs (`FielderAttrs::ground_decl_sigma`). It gives 0.5 m at 30 m, 1.9 m at 60 m, 3.4 m at 80 m, 5.3 m at 100 m.

He has the cue while the ball rolls, and on a hop from where it last touched the ground (σ grown by his velocity read along his line of sight times the time since). It is weighed against the snapshot read's depth σ by precision, and the play's fixed depth draw is applied at the result. Nothing else in the read changes; no new draws.

*Why not a track started at the bounce (dropped 2026-11-21).* That track diverged on far balls because its persistent error was a fraction of a range it believed short, and its fresh starts made the infielders' first half-second rougher (ground-ball hits .308 → .334). Here nothing carries a persistent error scaled by a believed range: the snapshot read applies its draw at the σ worked out at the true range (the world's draw, as before). The tracking read is untouched.

*Why not the range rate.* Successive looks at the angle also give how fast the range changes, but over T seconds of looks every 0.1 s its σ is about σ_g·√(1.2/T³): 3.7 m/s at 80 m after 1 s, against 1.8 m/s for his velocity read along his line of sight on a 30 m/s ball. It is worth using only after about 2 s, when the ball is near him.

*Infielders* use the same eye. One who first looks after a hard grounder's bounce now reads its depth to about 0.30 m at 25 m (0.75 m before). His first look on a grounder still in the air, and every tracked read, are unchanged.

## 4. The two plays, before and after

- **Seed 65147 play 10** (frozen fixture): the right fielder's worst read after the bounce, along his line of sight, 31.9 m → 4.2 m. He still fields it (`seed_65147_play_10_an_outfielder_takes_a_grounder_through_the_hole_both_left_to_the_other` passes).
- **Seed 1 play 66**: the centre fielder's depth draw is about −3.4σ. Before, his read put the chopper 107 m off, behind him, and he turned for the wall; now it is 15 m off (4.4 m σ at 92 m), and he takes a backup job toward the left side.

## 5. Measurements (standard run; the ledger row has them all)

Main → this branch, seeds 0 [1000]: runs 4.09 → 4.10 [4.37 → 4.32]; GB hits .307 → .309 [.312 → .308]; LD .670 → .666 [.662 → .659]; FB .098 → .102 [.101 → .102]; HR/PA 2.75% → 2.79% [2.84% → 2.82%]; reached on BIP 33.7% → 33.7% [33.8% → 33.7%]. All within the sets' spread; ground-ball hits moved +.002 [−.004], opposite signs, and did not rise.

From the probe, 300 games: chasers ever more than 20 m off in depth 11.8 → 0.42 [11.2 → 0.39] a game (the perfect-depth bound: 0.38 [0.39]), more than 10 m 17.6 → 1.24 [16.7 → 1.23]; median depth error while chasing 0.5 s after the bounce 15.5 → 2.1 m, at 1 s 9.9 → 1.4 m; ground-ball triples 108 → 89 [93 → 89], doubles 362 → 355 [371 → 334]; an outfielder secures a ground-ball hit 3.92 → 3.84 s [3.89 → 3.85 s] after contact.

## 6. Tests, and the mutations each fails under

- `the_range_of_a_ball_on_the_ground_follows_its_angle_below_his_eyes` (live.rs): `ground_range_sigma` against the geometry itself (h / tan(atan2(h, d) ± σ)), 2–110 m, to 0.1%. Fails with d²/h for (d² + h²)/h (at 2 m, 2.4e-6 against 4.1e-6).
- `a_ball_on_the_ground_is_read_where_his_eye_puts_it` (live.rs): a grounder up the middle, read by the snapshot read by the shortstop and the centre fielder with their depth draw at ±1σ and ±2σ: (1) the ground cue is absent before the first bounce, the time since the last bounce on a hop and 0 rolling, against the world's path; (2) while it rolls, the angle below his eyes of where he reads it is within draw × σ_δ (15% for the geometry's curvature at 2σ) of the true one. Fails with no cue (the shortstop at 30 m, 9.2′ against 6.9′), with the cue's σ doubled (7.4′), and with no cue on hops (part 1).
- `seed_65147_play_10_the_right_fielder_reads_the_chopper_on_the_grass_within_a_few_metres` (tests/throw_choices.rs): old against new on the frozen play, the right fielder's worst read after the bounce under 8 m (4.2 m; 31.9 m with no cue, which fails it).

`cargo test --workspace`: 578 passed, 0 failed, after a touch.

## 7. Open

- The cue as a measurement of the tracking read's looks (a tracked fielder after the bounce is already a median 1.6 m off at worst in 3 s, bounce-track note §10, so little to gain), and its range rate once the ball is near.
- The doubt an untracked fielder's call weighs (`landing_depth_sigma`, a catch in the air only), the runners' reads of a ball on the ground and the receiver's read of a throw on the hop do not use it.
- σ_δ 3′ is unsourced as a number; the cue itself is sourced. A grade-20 fielder has 4.6′.
- A persistent draw near its 4σ truncation still misplaces a far ball by about 15 m (seed 1 play 66), as the snapshot read of a fly would; such draws are rare (one fielder-play in about 1,500).
