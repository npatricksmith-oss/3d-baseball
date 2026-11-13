# The fielder's bat read, derived (2026-11-13)

**Band and mechanism.** No calibration knob was turned toward a target. By owner decision (2026-11-13), the σ of a fielder's read of the bat's angle at contact (`Anticipation`, agent-models §5.10 and §10) is now derived from what a fielder can perceive, and this note records how the run environment moved. The earlier values (15° at grade 50, 7.7° at 80, 29° at 20) were set for the first anticipation model, which read the ball's spray. They were never re-derived when the Anticipation review switched the read to the bat's angle (docs/calibration/2026-11-12-anticipation-review-fixes.md, "Not done").

## The derivation (agent-models §5.10, "How well the bat can be read")

- **(a) Timing dominates.** At contact the bat's horizontal angle sweeps at 1,915°/s (mean of 2,835 fair balls, sd 143°/s; `examples/bat_read_geometry.rs`, 48 games), so the read error is 1,915°/s × σ_t, where σ_t is the fielder's error in pairing the bat's angle with the moment of contact. That is the task of judging where one moving object was at the moment of a cue, measured against stationary references:
  - Linares, Holcombe & White (2009, *J. Vision* 9(13):9) report an SD of 70 ms. Their Exp. 1 put the cue away from the object (75 ms). Their Exp. 8 put it on the object's own path (40 ms).
  - Brenner & Smeets (2010, *Vision Research* 50:1101) report synchrony and temporal-order SDs of typically 40–50 ms, and 20–30 ms at best.
  - Brenner & Smeets (2015, *J. Vision* 15(3):8) give 20 ms at best for simple visual timing tasks.

  The anchors follow from these. Grade 50 is 40 ms (`BAT_READ_TIME_SIGMA_50_S`). The existing Anticipation ratio ρ = 1.25 then gives 20.5 ms at grade 80 and 78 ms at grade 20, and both are published values too. Interception-level precision (6–7 ms) and the precision of aligning bars that move together (about 5 ms) do not apply: the first is a guided action, and the second is a judgment of relative position.
- **(b) Viewing geometry is minor.** Take the resolution limit of normal vision (1 arcmin). At the standard positions the error from distance and edge-on viewing is 0.7° at first base, 5.4° at shortstop and 11.5° in centre field (median view). That changes the grade-80 σ by at most 4% and the grade-50 σ by at most 1%. One σ per grade is therefore kept, and the viewing term is not built.
- **(c) The pre-reaction window.** The read is one judgment of one look, made from the contact instant before the reaction gate opens. Sound reaches centre field too late (0.28 s) to steer a jump that begins at contact.

**Resulting bat-angle read σ:** 39° at grade 80, 77° at grade 50, 150° at grade 20, and 6.6° at the lab grade 160. These replace 7.7°, 15°, 29° and 1.3°. The bat's face varies only 9.3° between swings. Even an elite fielder's read therefore barely narrows the spray: the cue's σ for the spray is 56° at grade 80 and 106° at grade 50, against a 22° prior from the pitch location. The back-half policy (a read that puts the ball behind the batter tells him nothing, and he stays set) now fires on about 10% of reads at grade 80, 39% at 50 and 66% at 20.

## Measured (1,024 games, `sim-cli games --n 1024 --seed 5`, every fielder at one grade; base `2e715df`)

| Mode | Runs/team-game before → after | GB hit | LD hit | FB hit | PU hit |
| :--- | :--- | :--- | :--- | :--- | :--- |
| No jump | 5.57 → 5.57 | .412 → .412 | .726 → .726 | .133 → .133 | .046 → .046 |
| Anticipation 20 | 5.48 → 5.59 | .409 → .411 | .719 → .719 | .133 → .135 | .045 → .044 |
| Anticipation 50 (default) | 5.49 → 5.52 | .410 → .411 | .723 → .720 | .132 → .134 | .045 → .042 |
| Anticipation 80 | 5.33 → 5.54 | .407 → .414 | .718 → .723 | .129 → .134 | .045 → .045 |
| Anticipation 160 (lab) | 5.40 → 5.36 | .409 → .405 | .718 → .717 | .128 → .131 | .045 → .044 |

The old note's "no jump 5.34" was stale. On this base the no-jump run is 5.57; it reads nothing and is unchanged to the digit. With the derived read, anticipation on the 20–80 scale is worth almost nothing against no jump: 0.05 runs per team-game at grade 50, 0.03 at grade 80, and −0.02 at grade 20. The grades no longer rank in order, because the differences between them are within whole-game noise; later pitches diverge, so a few hundredths mean nothing. Only the out-of-scale grade 160 still saves about 0.2 runs; its read (6.6°) is close to the old grade-80 value. The default run environment rises 0.03 runs per team-game, to 5.52.

## What this says

The bat's angle at the contact instant is not a cue a human can read well enough to matter: the bat turns about 77° in one grade-50 timing error. The honest jump left on the 20–80 scale comes from the pitch-location prior. If anticipation is to matter, it needs a cue a fielder can actually perceive: for example, the swing's timing visible before contact (early or late relative to the pitch), or the contact point's depth seen relative to the batter. Adding such a cue is separate work; this change adds none. Retuning the jump rule to recover the old value would be a fudge.

## Not done

- The spray-on-face fit (`SWING_SPRAY_*`: 5.2° + 1.36 × face, 18.6° scatter, from 96 games on an older base) was not refitted. On this base, 48 games give 4.2° + 1.33 × face with 19.5° of scatter, which is close enough, and the cue is now too weak for the difference to matter.
- With read errors this large, the back-half policy now decides whether a fielder leans much more often than it did, and it was written for rare large misreads. A fielder who knows his read is that poor could instead ignore the cue and lean on the prior alone. That is a policy choice for the owner and is unchanged here.
- The viewing term (b) is documented but not built. It would matter only if the timing term were sharpened.
