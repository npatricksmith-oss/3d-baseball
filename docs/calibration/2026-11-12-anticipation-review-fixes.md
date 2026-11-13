# Anticipation after the Codex review fixes (2026-11-12)

**Band and mechanism.** No knob was changed. This note records how the run environment moved when the fixes to the Codex anticipation review (docs/reviews/2026-11-12-codex-anticipation-review.md) changed what fielders know before they read the ball (agent-models §5.10). Per the owner's close-enough rule, nothing was tuned to recover the old jump.

## What changed

1. **The bat, not the ball (review #1).** Before his reaction gate a fielder used to read the batted ball's true outgoing direction plus noise. That read carried the collision's result. Now he reads the bat's horizontal angle at the actual contact instant, with error σ `swing_sigma`: 15° at grade 50, 7.7° at grade 80, 29° at grade 20. He maps the angle to a spray belief with his own fitted line: spray ≈ 5.2° + 1.36 × face, with 18.6° of scatter (r = 0.57; 5,386 fair balls in 96 games, `examples/spray_by_bat_angle.rs`). The face angle varies only 9.5° between swings, so a 15° read error swamps most of its signal. The cue's σ is 27.6° at grade 50, weaker than the 22° pitch-location prior; 21.3° at grade 80; and 44° at grade 20. Fielders therefore commit less, and less often correctly, than with the old privileged read.
2. **Plans before the gate count the rest of the jump (review #2).** Position and velocity are stepped through the remaining jump interval. This changes only fielders who are leaning, so the no-jump run is identical to the digit.
3. **Circular fusion and the back-half policy (review #3).** Fair balls are unaffected in practice. Only a large misread puts the bat cue behind the batter.

## Measured (1,024 games, `sim-cli games --n 1024 --seed 5`, every fielder at one grade)

| Mode | Runs/team-game before → after | GB hit | LD hit | FB hit | PU hit |
| :--- | :--- | :--- | :--- | :--- | :--- |
| No jump | 5.34 → 5.34 | .410 → .410 | .723 → .723 | .118 → .118 | .005 → .005 |
| Anticipation 20 | 5.28 → 5.30 | .406 → .408 | .721 → .722 | .121 → .119 | .003 → .005 |
| Anticipation 50 (default) | 5.18 → 5.27 | .401 → .409 | .725 → .722 | .118 → .122 | .004 → .005 |
| Anticipation 80 | 5.02 → 5.16 | .384 → .406 | .718 → .719 | .113 → .117 | .002 → .004 |

The value of anticipation roughly halves. Against no jump, grade 50 now saves 0.07 runs per team-game (it saved 0.16), and grade 80 saves 0.18 (it saved 0.32). The grades still rank in the right order. Almost all of the old gain was on ground balls, and it came from the privileged read. The honest cue gives back most of it: average fielders field ground balls for a .409 hit rate, against .410 with no jump. The default run environment rises 0.09 runs per team-game, to 5.27. This is whole-game comparison: later pitches and opportunities diverge, so differences of a few hundredths are noise.

## Not done

- No re-tuning. If the owner wants anticipation to matter more, the honest levers are physical: a sharper bat read (the `swing_sigma` scale is now degrees of bat angle, and §10's values were set for the old spray read), or more cues (depth from the swing plane, pitch type). The jump-commitment rule should not be raised to compensate.
- The bat-angle fit is marginal: it ignores pitch location, so fusing it with the location prior counts a little information twice. A joint fit (spray on face and location) would be the cleaner belief.
