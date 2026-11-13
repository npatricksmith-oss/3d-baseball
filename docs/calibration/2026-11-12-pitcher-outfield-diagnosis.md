# Pitcher in the outfield: diagnosis (2026-11-12)

Diagnosis only; nothing in `src/` is changed. Code references are to HEAD `638871c`
(`sim/crates/bb_game/src/live.rs`). TASKS.md item: "Pitcher in the outfield (sim bug, open)".

## Symptom

On fly balls and hits to the outfield the pitcher ends the play 50–100 m from home, usually
still at full speed (7.9 m/s). He is not the chaser and is not at a base or a backup spot.
Seed 7: play 1 "Fly out to right" ends with him at (54, 55); play 195 ends with him at (21, 86).

## Reproduction

```bash
export PATH=$HOME/.cargo/bin:$PATH
cd sim
cargo build --release -j 2 -p bb_game --example pitcher_roam
./target/release/examples/pitcher_roam 7 1          # seed 7: lists the 7 flagged plays, incl. 1 and 195
./target/release/examples/pitcher_roam 1 200        # 200 games, seeds 1..200: summary on the last line
./target/release/examples/pitcher_roam 7 1 all      # every ball in play, flagged or not
```

`examples/pitcher_roam.rs` (committed with this note) steps `Game::new(seed, true)` pitch by
pitch and reads the pitcher's recorded track from `PlayViz::actors` (id `0x81` in the low
byte). A play is flagged `FAR` when he ends more than 30 m from the mound and more than 30 m
from every base. Plays are numbered by `PitchStep::play`, the same numbering TASKS.md uses: seed
7 play 1 is "EV 102 mph, LA 36°, spray 28° | out (FlyOut)" with the pitcher ending at (54, 55),
and play 195 is "EV 103 mph, LA 36°, spray 10° | out (FlyOut)" ending at (21, 86).

The role trace below came from temporary instrumentation in `simulate_live_inner`, gated by an
environment variable and since discarded. After each sub-step (`assign`, `recall`,
`update_thrown`, `try_captures`, `holder_act`, `cover_home_again`, `update_relay`) it printed
the pitcher's `(role, backs, target)` whenever any of them changed. At the end of each play it
printed his final role and the sub-step that last changed it.

## Trace of the two cited plays

Seed 7, play 1 (bases empty, fly to right-center, RF chasing):

| t after contact | set by | role | backs | target | pitcher at |
|---|---|---|---|---|---|
| 0.25 s | `assign` | Idle | None | (62.7, 87.9) | (0.0, 18.3) |
| 3.57 s | `recall` | Idle | None | (62.8, 53.0) | (13.9, 33.8) |
| 4.82 s | `recall` | Idle | None | (74.3, 65.9) | (22.4, 37.9) |
| end | | Idle | None | (74.3, 65.9) | (54.3, 55.0), 7.9 m/s |

Seed 7, play 195 (bases empty, fly to center): `assign` at 0.25 s gives Idle, target
(15.1, 104.1). Two `recall`s move the target to about (32, 106). At 9.52 s a throw gets
away (`update_thrown` → `set_free` → `assign`), so his target becomes his own intercept of
that loose ball, (−38, −40). The play ends with him at (21.4, 86.5).

His target is never a base or a backup spot. It is **his own intercept of the ball**.

## Root cause

Three pieces combine.

1. **`assign_among` writes every candidate's intercept into `target`** (l.903,
   `self.f[i].target = pt;`, inside the loop at l.885–906). This is intended for the chaser,
   and it keeps the ground-ball break working (that loop reads its own copy, `own`, l.927).
2. **`cover_roles` leaves a fielder with no job as `Role::Idle` without resetting `target`**
   (l.1247–1253). The comment there says "`assign_backups` gives most of them one". The line
   `let _ = tgt;` (l.1252) is left over from 31d57bb. Before that commit this branch gave
   outfielders a backup target; after it the branch writes nothing, so the intercept from
   step 1 stays.
3. **`assign_backups` has no job for the pitcher on an outfield ball with the bases empty**
   (l.1138–1143). The jobs are `(2, [far_of, 0, 7])` and `(1, [1])`. The far-side outfielder
   is almost always free, so he takes second and the pitcher's turn never comes. With a
   runner on, the pitcher is first in line for third or home (l.1132–1137), which is why the
   bug is almost entirely a bases-empty bug.

`fielder_step` (l.546–608) drives every fielder toward his `target` whatever his role. For a
non-chaser, `early` is true (l.574), so he runs at top speed and brakes only near the spot.
The spot is his intercept, which keeps moving away from him as the ball carries.

The bug also feeds itself. Once a second, `recall` (l.864–880, called from l.2189) re-calls
among the four fielders nearest the chaser's target. The pitcher has been running toward the
ball, so he is often one of those four. Each re-call then refreshes his intercept target
(l.903). That is why `recall` is the last writer in 60% of the flagged plays. A loose ball
after a deflection (`try_captures` → `set_free`, l.1418–1428) or a throw that gets away
(`update_thrown` → `set_free`) calls `assign` again and restarts the same cycle.

The relay job, `back_up_throw`'s fallback (l.1181–1194) and `backup_spot` (l.403) are **not**
involved: in 200 games they account for one flagged play.

## Frequency (200 games, seeds 1–200, 14,626 balls in play, HEAD 638871c)

The pitcher ended more than 30 m from the mound and every base on **2,006 balls in play
(13.7%)**. Median distance from the mound: 63 m (max 103 m); 1,173 plays end more than 60 m
from it. Median end speed: 7.9 m/s; 84% end at 7 m/s or faster, i.e. still sprinting.

| Pitcher's job at the end of the flagged play | Last set by | Plays | Share |
|---|---|---:|---:|
| Idle, target = his own intercept | `recall` | 1,201 | 59.9% |
| Idle, target = his own intercept | `assign` (at contact) | 447 | 22.3% |
| Idle, target = his own intercept | `try_captures` → `set_free` → `assign` | 301 | 15.0% |
| Idle, target = his own intercept | `update_thrown` → `set_free` → `assign` | 28 | 1.4% |
| Chase (he is the primary) | mostly `update_thrown` | 28 | 1.4% |
| Backup of third | `holder_act` → `back_up_throw` | 1 | 0.0% |

By base state: of 7,937 balls in play with the bases empty, 1,987 are flagged (25.0%). Of
6,689 with runners on, 19 are flagged (0.3%). By outcome: 933 fly outs, 916 hits or other
balls in play, 116 tag plays. On 23 plays the pitcher himself secured the ball more than 30 m
from the mound.

The 28 `Chase` rows are a separate, minor issue. They are balls hit backward (spray
160–180°), where the pitcher becomes the primary chaser of a ball rolling behind home
(targets near y = −85). They are not part of this bug; they are listed so the fix is not
expected to remove them.

**The same leak affects other fielders.** Over 50 games (3,712 live plays) of the same
instrumented run, these fielders ended Idle and still moving faster than 5 m/s: P 41% of
plays, CF 30%, RF 15%, 2B 14%, LF 10%, SS 4%. For outfielders this mostly looks like
charging the ball as an informal backup, so it is less visible. For the pitcher it is the
bug the owner saw.

## What real baseball and the spec say

- `agent-models.md` §5.7, "Backups": "every fielder with no other job backs up the play …
  (the pitcher behind third or home, the far outfielder behind second, the catcher behind
  first with the bases empty)". The full list is in
  `docs/calibration/2026-11-11-owner-watch-notes.md` §Backups. That list has no pitcher job
  for "Outfield hit, nobody on", and the code matches it. The gap is in the spec's table as
  well as in the code.
- Standard defensive assignment charts never send the pitcher into the outfield. With
  runners on, he backs up third (runner on first) or home (runner in scoring position); the
  code already does this. With the bases empty, he moves off the mound toward the infield.
  On a ball in a gap or down a line, where the batter may try for three, he backs up third.
  On a routine single he stays in the middle of the diamond, ready to back up a throw. On a
  ball to the right side of the infield he covers first.
- Nothing in §5.7 says a fielder with no job runs toward the ball. "Idle" should mean he
  holds his position or makes a short move toward the play, not a chase.

## Proposed minimal fix (not applied)

**Part A, the root cause (`cover_roles`, l.1247–1253).** When a fielder is left `Idle`, set
his target to where he would stop if he braked now (`self.f[i].target = stop_point(&self.f[i])`),
instead of leaving his intercept there. Then remove the dead `let _ = tgt;`. This covers
`assign`, `recall`, and every `set_free` re-assign, because they all go through
`assign_among` → `cover_roles`. The ground-ball break is unaffected (it reads `own`, saved
before `cover_roles`). The relay check reads `t_int` and `belief`, not `target`, so it is
unaffected too.

**Part B, give the pitcher his job (`assign_backups`, outfield case with the bases empty,
l.1140–1142).** Add a pitcher job after the existing two: `jobs.push((3, vec![0]))`. He
backs up third on the line from the fielding point, the base the batter-runner would go for
on an extra-base hit. Third is chosen because it is the one place §5.7 already names for him
("behind third or home"). Moving toward second would be just as defensible on a routine
single. Record the choice as "Proposed" in §5.7 and in the owner-watch-notes list (outfield
hit, nobody on: "the pitcher backs up third"), since it is an owner-visible choice.

Either part alone removes almost all of the symptom. Part A alone leaves him standing on
the mound: the first assign comes at his reaction time, before he has moved. That is
believable but passive. Part B alone fixes the pitcher but not the other Idle fielders.
Apply both.

**Expected side effects to measure.** Outfielders who now drift toward balls they do not
chase would hold their positions instead (an Idle outfielder still running at the end: CF in
30% of plays, RF 15%, LF 10%). Rerun `sim-cli games --n 1024` and compare runs per team-game and
BABIP by batted-ball type against the handoff numbers (5.22 runs, GB .399, LD .721,
FB .116). A small shift is expected. Under the close-enough calibration rule this needs no
retuning unless it moves out of band.

## Test that fails on current code

A unit test in `live.rs` `mod tests`, in the style of `idle_fielders_back_up_the_play`, using
the `fixture`, `input`, `jobs` and `behind` helpers and that test's `batter` closure:

```rust
// Fly to right-center, nobody on (seed 7, play 1): the pitcher has a job in the infield
// and is not sent at the ball.
let fx = fixture(&[]);
let inp = input(&fx, vec![batter(&fx)]);
let mut st = fx.st.clone();
let mut sim = new_sim(&inp, &mut st);
let at = Vec3::new(62.0, 88.0, 0.0);
jobs(&mut sim, 8, at);
assert!(behind(&sim, 0, 3, at), "P: {:?} {:?} {:?}", sim.f[0].role, sim.f[0].backs, sim.f[0].target);
```

On current code `jobs` leaves the pitcher `Idle` with `backs: None`, so this fails. It needs
Part B.

A test for Part A goes through `assign_among`, because `jobs` sets only the chaser's target.
Set every fielder's target to `at`, which is what `assign_among`'s loop does, then call
`cover_roles` and `assign_backups`. Every fielder left `Role::Idle` must have a target within
a few metres of where he stands (`(target - pos).norm() < 5.0` for a fielder at rest). On
current code the Idle fielders keep `at`, 60–90 m away, so this also fails.

An end-to-end check is the example itself. After the fix, `pitcher_roam 7 1` should flag
none of plays 1 and 195. `pitcher_roam 1 200` should report fewer than 0.5% flagged, leaving
roughly the 28 balls-behind-home `Chase` plays.
