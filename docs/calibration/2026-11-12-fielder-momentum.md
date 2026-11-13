# Fielder momentum and the planner bugs it exposed (2026-11-12)

Owner direction (2026-11-12): fielders should get a jump on the ball by anticipating from pitch location and batter handedness, with a rating for it (agent-models §5.10, next). A wrong guess must cost something, which needs momentum. Turning on honest momentum showed how much the fielding planner had leaned on fielders who could turn and reverse instantly at full speed. The owner chose to fix the movement and planning first ("full momentum first").

All numbers: `sim-cli games --n 512 --seed 2`, runs per team-game and hit rate by batted-ball type (provisional lab ruling). MLB: about 4.5 runs; GB .24, LD .68, FB .13, PU .02.

## 1. Sequence

| Step | Runs | GB | LD | FB | PU |
| :--- | ---: | ---: | ---: | ---: | ---: |
| Start (HEAD `1a0eaef`, instant turns) | 5.14 | .385 | .716 | .141 | .017 |
| Speed counts only toward the target (projection) | 6.31 | .408 | .784 | .207 | .042 |
| + ground-ball break rule fixed (bugs 1, 2) | 6.26 | .405 | .784 | .208 | .046 |
| + calls on balls in the air, re-call set, stickiness (bugs 3–5) | 6.09 | .412 | .783 | .153 | .011 |
| (same fixes, instant turns, for reference) | 4.94 | .393 | .714 | .097 | .002 |
| Vector velocity, brake 6 m/s², a_lat 7 m/s² (first form) | 8.24 | .461 | .813 | .310 | .053 |
| Braking and turning take what is needed up to the limits | 7.58 | .456 | .803 | .257 | .033 |
| + plant after a catch; foot on the bag for throws | 7.24 | .451 | .798 | .255 | .032 |
| + typical spin by launch angle (bug 6) | 5.71 | .441 | .703 | .091 | .004 |
| + velocity read 0.06 → 0.04, OF position read 2.5 → 3.0, receivers weigh the aim | **5.29** | .400 | .727 | .110 | .005 |

Per-game time is unchanged (about 260 ms on one thread for both this build and HEAD on a loaded laptop, `examples/prof_game.rs`).

## 2. Bugs found (planner, not abilities)

1. **Infielders broke for pop-ups as if they were grounders.** The "everyone breaks on a ground ball" rule tested whether the fielder's read had the ball coming down within 25 m. An infield pop-up does too. Now the ball must come down within 25 m within 1 s of contact (`GROUNDER_BOUNCE_S`).
2. **A breaking infielder ran at his base.** The rule gave him the chase role but kept the target the cover step had just set (the bag) until his next replan, up to 0.4 s later.
3. **Fly balls went to whoever had the earliest intercept.** On a fly everyone's earliest catch is when it comes down, so the call was a coin toss decided by noise, and the outfielders' 0.15 s bonus handed shallow pop-ups to outfielders who had misjudged them. Calls on balls in the air now go to confident callers (0.3 s to spare): outfielders first, then the most time to spare.
4. **The re-call looked at fielders near the ball's current position.** For a pop-up at 2.4 s that is 30 m above the infield, so a first baseman standing on his bag took the ball from the second baseman under it. It now looks near where the chaser is going.
5. **Calls were traded on noise.** A re-call now changes the chaser only for a higher tier or 0.3 s more to spare.
6. **Fielders assumed 1,800 rpm backspin on every rising ball.** The contact model gives fly balls 3,300–6,500 rpm (median by launch angle), so flies carried past every prediction and the target crept back all flight. Spec §5.1 already said to use the contact model's typical spin for the launch angle. Now a belief table does (`typical_batted_spin_rpm`, from `examples/batted_spin.rs`).

Free movement also hid two smaller artifacts: after a running catch the fielder went back to his planned spot before throwing (now he plants where braking stops him), and a first baseman stepped off the bag to meet a throw and then back on (now he keeps his foot on it for any throw he believes is within reach, §5.8).

## 3. Mechanism and band for each refit (calibration-targets §2.3–§2.4)

- **Velocity read error 0.06 → 0.04** (`READ_VEL_FRAC`, named; was a literal). Band: ground-ball hit rate (B2 BABIP by type). Mechanism: with momentum, every replan that moves an infielder's target is a turn at speed. A first-read direction error of 3.4° on a 35 m/s grounder put his early target 2–3 m off. Sweep: 0.06 → GB .441, 0.04 → .394, 0.02 → .371.
- **Outfield position read 2.5 → 3.0 m at 30 m.** Band: fly-ball hit rate. Mechanism: once the spin belief was fixed, flies were too easy (.081). Sweep at velocity read 0.04: 2.5 → .081, 4.0 → .190; 3.0 gives .110. The owner had accepted 3.0 on 2026-11-11 before the left-field index fix.
- **Receivers weigh the aim point** (`THROW_MISS_FRAC` 0.05). No band; a mechanism fix. A catcher's first read of a 57 m relay throw sent him 17 m behind the plate (test `a_throw_beyond_the_arm_goes_through_the_relay_man`).

## 4. Tried and not kept

- A 0.15 s set-up margin as a hard planning requirement (agent-models §5.2 wording): +0.5 runs, because it rules out plays the fielder can make.
- A chaser who cruises at the pace that gets him there just in time: +0.9 runs, because he leaves himself no slack for read changes.
- Shedding sideways speed at a_lat in the target's frame: the fielder orbits a spot inside his turning circle (+2 runs). Replaced by braking and turning limits on the velocity itself.

## 5. Still open

- Ground balls remain the excess (.400 vs .24). Infield hits are 20% of fair grounders against MLB's 3–4%, most on high choppers: the contact model's launch-angle spread (quartiles −15/5/25° vs MLB ≈ −2/13/28°).
- Turning is still the largest cost. At a_lat 50 m/s² (effectively free) runs would be about 4.4; at the spec's 7 m/s², 5.3. Remaining chaser turns come mostly from replans that move the target (read updates), then from fielders who become the chaser while running elsewhere. The atlas perception of §5.1, with a launch-state error that shrinks smoothly, should reduce them.
- Diagnostic tools added: `examples/spray_by_location.rs`, `batted_spin.rs`, `bip_outcomes.rs` (one line per ball in play, for diffing two builds), `trace_play.rs` (one play's tracks and facts), `pu_hits.rs`.
