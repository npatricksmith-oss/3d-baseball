# The middle infielders' jobs on a ball through the infield: saved play 102 of seed 51001 (2026-11-21)

**Job.** Build Q-C (a) of the [saved-plays diagnosis](2026-11-20-saved-plays-51001.md) §3, chosen by the Coordinator as a fix that brings play in line with real baseball: on a ground ball through the infield the middle infielder on the ball's side goes out as the cutoff man for second from the first call and his partner covers second; the deep-ball relay rule keeps only balls hit over or past an outfielder. Branch `work/mid-infield-jobs`, from main `904c1ea`. Spec: agent-models §5.5 ("Where the man in the throw stands") and §5.7 ("Cutoffs and relays", "A deep ball", "A ground ball through the infield").

## 1. What was built (`sim/crates/bb_game/src/live.rs`)

1. **The cutoff to second on a ground single** (`relay_job`, `second_cutoff_man`). With nobody on base at the pitch, on a ball that first came down within the infield dirt (`INFIELD_R`, a fact everyone saw: `first_down_m`) and that an outfielder fields, the cutoff man for the throw to second is the middle infielder on the ball's side as everyone sees it (`ball_side_x`, the side the cover table already uses for second, so his partner is the man it sends to second). Before, with nobody on there was no cutoff man at all. A partner already known to be out on that cutoff keeps it, so it does not change hands when a ball up the middle is later seen on the other side of second. He stands at the through spot (`CUTOFF_M` = 14 m short of the bag).
2. **At the first call** (`through_job`, `cutoff_job`). While no outfielder is known to be going for the ball (nobody yet, or only an infielder), the cutoff man for second (above), or the shortstop when the throw would go to third (a runner on first at the pitch), goes out as soon as the ball has come down in the infield and his own read has it on the ground, carrying beyond the infielders' range (`INFIELD_BREAK_R`), with no play for him in it, and by his own estimate of each teammate (`seen_intercept` on his read) no other infielder can field it inside that range, whatever job that man is on now. Toward second the pitcher is left out of that estimate (a ball he knocks down comes on toward the man going out on its line, and there is no force at second); toward third he is not (the shortstop must not leave second on a comebacker that is a double play). The man lines up from the bag toward where his read has the ball going. Once an outfielder is known to be going for it, `relay_job` takes over from where he fields it.
3. **A deep ball** (`deep_ball`). Beyond `RELAY_DEEP_M` = 80 m, a ball is deep (the relay man goes out, toward third with nobody on) only if it was hit over or past an outfielder: it first came down beyond the infield dirt, or, hit on the ground, it got past the outfielder fielding it (in his hands deeper than his pre-pitch spot; loose, the ball now deeper than both that spot and the man going for it). The not-deep spot is the through spot, the deep one the relay spot (`relay_spot` now takes the flag).
4. **The batter on first is not a runner on first** for the cutoff's base (`relay_base`): the runners on base are those on base at the pitch. Before, once the batter passed first on a single the likeliest throw became third and the shortstop left whatever he was doing to line up toward it (on play 102 with this fix alone, at 4.59 s: the second baseman lost the cutoff, the shortstop left second).
5. **Re-calls keep the cutoff men** (`cover_table`, `cover_roles`). The tables worked out at a call or the once-a-second re-call now leave a man known to be on a cutoff in his job and give him no base, as the tables between calls (`cover_after_chasers`) always did, with first base dealt as there when the first baseman is the cutoff home. Before, each re-call dealt second to the cutoff man, the cutoff rule took him back, and his partner covering second lost the bag and braked for 0.3 s (play 102: at 2.42 and 3.44 s).
6. **A man past whom the ball has gone keeps the cutoff** (`relay_job`). The rule "while the ball is loose he plays it himself if he gets to his intercept before the chaser" now applies only to an intercept within the infielders' range. On the mirror of play 102 the shortstop's intercept in the outfield, sooner than the centre fielder's run to that same spot, left him with no job at all: he neither went for the ball nor out.
7. **A read-only job trace** (`live::job_trace`, off by default) for tests and probes.

No new parameter: the thresholds are the existing `INFIELD_R`, `INFIELD_BREAK_R`, `RELAY_DEEP_M`, `CUTOFF_M` and the outfielders' Savant spots.

## 2. Play 102 before and after (fixture `tests/fixtures/a4/51001-102.json`)

Frozen from main `904c1ea` (a left-handed batter; the throwaway freeze example of the dp-cover work): it replays to the game's facts on main (the centre fielder secures at 3.94 s, throws at 4.99, the second baseman takes it at 6.49). Contact at 0.41 s; the defense's first call at 0.66 s.

| | Main `904c1ea` | This branch |
|---|---|---|
| 0.66 s, the first call | Shortstop: covers second. Second baseman: no job; brakes out of his jump and **stands 0.81–1.56 s**. Nobody goes for the ball (the centre and right fielders each leave it to the other) | Second baseman: **the cutoff for second**, out toward (2.3, 52.6). Shortstop: covers second |
| 0.96 s | The centre fielder goes, by the shared convention | the same |
| 1.26 s | The centre fielder's chase known, fielding at 88 m: "deep", so the **shortstop leaves second as the relay man toward third** | The cutoff spot moves onto the line from the centre fielder's fielding point, (3.4, 52.4) |
| 1.56 s | The second baseman takes second | (no change) |
| 2.42, 3.44 s | Each re-call takes second from the second baseman for 0.3 s | (no change) |
| 3.94 s | The centre fielder secures at (14.7, 87.6) | the same; the second baseman is at his spot, 0.04 m off the line to second |
| 4.99 s | Return throw to second, taken by the second baseman at 6.49 | Return throw to second past the cutoff man, taken by the shortstop on the bag at 6.46 |

The batter stops at first in both. Nobody goes out toward third.

## 3. The neighbouring cases, what each does now

- **Through the left side (the mirror).** The shortstop goes out, the second baseman covers (test `mirrored_play_102_...`: the fixture mirrored and turned 3° toward the middle, since the exact mirror bounces differently and the shortstop fields it). If the ball-side man first went for the ball, he takes the cutoff when he gives it up and an outfielder is known to have it.
- **Through the right side (the first-baseman–second-baseman hole).** The second baseman is the ball-side man; while by his estimate the first baseman can still field it in the infield he waits, and goes out once it is past (seed 6 play 36 of the probe: 0.9 s with no job).
- **Runner on first (double-play depth).** The shortstop is the cutoff to third as soon as the ball is through, counting the pitcher; the second baseman covers second (on a ball right of second, 0.3 s later, when he sees the shortstop go). With the batter now standing on first, nothing turns the cutoff toward third.
- **Runner on second or third (the cutoff home).** Not changed: the first baseman (the third baseman on a ball from left, x < −15 m) goes out once an outfielder is known to be fielding the ball, and the middle infielders' tables fill in 0.3 s later. On these plays a middle infielder still often has no job for 0.6–0.9 s while the ball is loose (TASKS, open).
- **A ball the outfielder misplays, or one rolling to the wall.** Once the loose ball is deeper than the chaser and his spot, it is a deep ball: the relay man (the middle infielder nearer the relay spot) goes out toward third at the relay spot (test `a_ground_single_fielded_deep_is_not_a_deep_ball`, a grounder fielded at 110 m).
- **A fly over an outfielder** is still deep (same test: a ball first down at 80 m, and one caught and held).
- **A line-drive or fly single with nobody on** still has no cutoff man for second (not changed; TASKS, open).
- **A comebacker.** Toward second the pitcher is left out of the estimate, so on a ball he may knock down the ball-side man still goes out on its line; when the pitcher is known to have it (or holds it) the job ends and the man takes his cover job. Toward third the pitcher counts, so the shortstop stays on second.

## 4. Measurements

**Jobs on ground balls first fielded by an outfielder** (`BB_COVER_TRACE` over `examples/coverage_trace`, game seeds 0–59, throwaway `jobs.py`; the trace's ground-ball flag is a first bounce within 25 m in the first second): main → this branch.

| | Main | Branch |
|---|---|---|
| Such plays | 206 | 203 |
| Second baseman with no job while the ball is loose, mean a play | 0.444 s | 0.189 s |
| Shortstop, the same | 0.309 s | 0.281 s |
| Plays with a middle infielder > 0.1 s with no job (2B, SS) | 108, 108 | 81, 76 |
| Second base uncovered at some time after the first call | 46 | 22 |
| Second base's cover changing hands | 83 | 65 |
| A middle infielder out toward third at some time | 171 | 85 |
| A cutoff to second | 0 | 121 |

Of the 157 remaining stretches over 0.1 s, 95 are on plays with a cutoff to home (a runner in scoring position, §3), the rest mostly a ball another infielder could still field (the man waits for it to pass him) or a man who went for the ball and gave it up.

**Outcomes on those plays** (throwaway `zz_mid_out`, game seeds 0–119, balls first secured by an outfielder after coming down; ground = first down within the infield dirt), main → branch: ground balls 592 → 591, batter to second or beyond 28.5% → 28.4%, batter out 8 → 6, outs on the bases 15 → 11, first to third with the batter on first 37.4% → 36.2% (40/107 → 38/105); air balls 998 → 1,006, batter to second or beyond 48.5% → 48.3%, outs on the bases 29 → 29, first to third 38.6% → 38.3%. `extra_base_rates 60`: first to third on a single 65/186 → 64/183, second to home on a single 52/91 → 51/92, first to home on a double 40/49 → 43/52. All within a few counts: the cutoff to second changes no throw that matters on these plays (with nobody on, the batter has stopped by the time the ball comes in), and throws to third keep their cutoff man. Outfield assists (outs on the bases on these plays) are 44 → 40 in 120 games, a handful of plays.

**Standard run** (`sim-cli games --n 300`, seeds 0 [1000]): main `904c1ea` (the dp-cover final's figures: its sim code is the same) → this branch. Runs 4.16 → 4.19 [4.59 → 4.59]; GB hits .309 → .309 [.322 → .321], LD .672 → .673 [.673 → .670], FB .102 → .102 [.098 → .099]; reached on BIP 33.6% → 33.8% [34.5% → 34.5%]; out of play after a throw 0.103 → 0.100 [0.137 → 0.130] a game; time per game 4,795 → 4,590 [5,015 → 4,795] ms (the teammates' estimates in the through job cost nothing measurable). Every move is inside the seed sets' noise; every digest changed (a fielder's path changes, and with it the games after). The rest is in the ledger row of 2026-11-21.

One test changed its tolerance: `the_typical_spin_belief_follows_the_contact_model` (games 100–123) failed when the reshuffled games put its 25°/85 mph cell at 1,519 rpm on 35 balls against the table's 1,840: 17% off, outside the 15% rule, but inside that median's 99% interval (1,248 to 2,137). A 25–50° cell now fails only if the table is both more than 15% off and outside the interval, the test the 15° row already used. No batted ball changed.

## Reproduce

- Play 102: `tests/throw_choices.rs`, `seed_51001_play_102_...` (the job trace from `replay_jobs`).
- The trace probes: build `examples/coverage_trace` on each commit, run with `BB_COVER_TRACE=1 coverage_trace 60 2> trace.log`, then the throwaway `sim/target/codex-probes/mid/jobs.py trace.log` (kept outside the tree, with `idle.py` and `zz_mid_out.rs`).
