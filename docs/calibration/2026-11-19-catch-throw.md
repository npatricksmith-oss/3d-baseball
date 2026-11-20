# The catch sets up the throw: how outfielders take the ball, what their long throws do, and the sources (2026-11-19)

**Job.** The owner's Q32 (agent-models §5.5, "The catch sets up the throw"): the fielder's momentum at the catch, relative to his target, decides the throw's time, speed and accuracy, and outfield throws go low through the cutoff man or to a relay man on deep balls. Step 1 measures the sim as built and finds sources (§1–§4); step 2 is the spec (agent-models §5.5); step 4 measures what the change moves (§5). §6 is the owner question it raised (Q33) and his answer. Branch `work/catch-throw`, from main `ab5c374`.

Source tags as in the [throw-race note](2026-11-19-throw-race.md): `read` (page text read), `snippet` (search-engine summary only), `unsourced`.

## Summary

- **Sim outfielders almost never wait under a fly.** Of 8,183 outfield fly catches in 600 games, 17.8% are made standing (under 1 m/s), and only 4.3% by a fielder who had stood still for half a second or more (1.1% for a second). The median speed at the catch is 2.5 m/s. Most catches are made moving across the line to second (45%) or away from it (21%); 16% coming toward it. Real outfielders camp under routine flies; the sim's "arrive under control" rule (agent-models §5.2) keeps them a doubt short of the spot, so they creep in to the catch.
- **The flat footwork rule charges most of these nothing.** On fly catches that led to a throw (5,495), the 0.2 s was charged on 19%: every catch moving away faster than 3 m/s, and a quarter of those moving across at 3 m/s or more (when part of it is away). A fielder caught moving away at 7 m/s owes 0.2 s, against 1.47 s to stop under the sourced braking limit.
- **Long throws are rainbows.** Outfield throws of 60–90 m leave at 14–18° and peak 5–9 m up, over any cutoff man's head. 17–28% come down short of the target (median 1.5–2.1 m short: speed noise near the aim), and the lateral miss is small (median 0.8–1.2 m, more than 3 m on 3–7%). Almost none go home: 12 of 7,038 such throws in 600 games; nearly all are returns to second with no play.
- **Relays have almost vanished** (9 in 600 games), as the relay review found.
- **Sources found:** a run-up toward the target adds about 7% of ball speed (1.6 m/s for 1.5 m/s more body speed, elite handball; read), with no loss of accuracy; a crow hop adds about 10% over a pitch (read). No measurement of an outfielder's plant or catch-to-release on a fly was found, nor an MLB share of throws through the cutoff by depth. One play (Reddick 2017, 276 ft, read) took 3.82 s from catch to the catcher.
- **After the build (§5, §6):** as decided in Q32 (the plant also charging motion across the line), the build added about half a run a team-game and raised the outfielders' mean pickup-to-release from 1.01 s to 1.46 s, against an empirical ≈1.2 s with the tag. The owner's Q33 defers the across term until fielders slow into the ball and round it (the next job). The final build, with the Opus review's and re-review's fixes, charges only motion away. Runs are 4.24 → 4.27 [4.47 → 4.57] and pickup-to-release 1.08 s. Tags from second and from third are at main's levels. Triples are 0.87 → 1.02 a team-game (MLB about 0.14), the plant on balls run down going away. 7 of 8 throws through a cutoff man reach the plate on the fly or one hop. The set-up behind a fly was built, dropped catches, and was taken out.
- **The physics puts the relay where the owner said.** A throw through a cutoff man about 45 ft from the plate, passing him at head height, carries home on one hop and costs 0.02–0.06 s over the plain aim. With each route's expected miss cost, the relay (with the sourced 0.75 s exchange) beats it from about 246 ft at 34 m/s, 276 ft at 37 m/s and 302 ft at 40 m/s (§4). The owner's figure is about 280 ft. (Revised 2026-11-20: the first build passed the throw past the cutoff man at the relay spot, and it rolled in.)

## 1. How outfielders take the ball (as built, main `ab5c374`)

A throwaway trace (`BB_CT_TRACE`, not committed; `sim/target/codex-probes/catchthrow/WIP-with-trace.diff`) logs every outfielder's catch of a batted ball and every outfield throw. The probe (`zz_ct_probe.rs`, from the relay job's probe) runs direct seeds 0–299 and 1000–1299. Speeds are the fielder's ground velocity at the catch; "toward", "across" and "away" are by which part of it is larger, along or across the line to the base; "standing" is under 1 m/s. "Settled" is how long he had been under 1 m/s before the catch (from his 0.1 s track).

**Fly catches** (both seed sets):

| Cases | n | Standing | Toward | Across | Away | Standing, settled ≥ 0.5 s | Median speed |
|---|---:|---:|---:|---:|---:|---:|---:|
| All outfield fly catches, against second | 8,183 | 17.8% | 16.3% | 45.4% | 20.6% | 352 (4.3%) | 2.5 m/s |
| Runner on third, fewer than two out, against home | 412 | 16.5% | 13.6% | 45.9% | 24.0% | 19 | 2.5 m/s |
| the same, 225–300 ft | 141 | 15.6% | 24.1% | 53.2% | 7.1% | 9 | 2.4 m/s |
| Runner on second, third empty, against third | 647 | 18.7% | 15.0% | 48.8% | 17.5% | 27 | 2.5 m/s |

By speed, all fly catches: toward 1–3 / 3–5 / 5+ m/s: 603 / 316 / 413; across: 1,953 / 915 / 844; away: 761 / 499 / 426.

**Outfield throws after a fly catch** (5,495; the first throw of the play, nearly all to second) and **after a pickup** (8,277):

| Approach | Speed | Fly: n | Fly: footwork charged | Fly: released after the catch | Pickup: n | Pickup: footwork charged |
|---|---|---:|---:|---:|---:|---:|
| standing | < 1 | 978 | 0 | 0.88 s | 245 | 3 |
| toward | 1–3 / 3–5 / 5+ | 421 / 201 / 251 | 0 / 0 / 1 | 0.91–0.93 s | 23 / 72 / 1,771 | 0 / 1 / 6 |
| across | 1–3 / 3–5 / 5+ | 1,323 / 603 / 564 | 2 / 80 / 146 | 0.89–0.97 s | 94 / 355 / 2,883 | 1 / 116 / 675 |
| away | 1–3 / 3–5 / 5+ | 506 / 342 / 306 | 145 / 342 / 306 | 0.93–1.10 s | 181 / 1,839 / 814 | 162 / 1,839 / 814 |

- Every catch-to-release is the 0.85 s outfield `Transfer` plus the turn and the 0.2 s where charged. None of these throws was hurried (no play), so none was off balance.
- Pickups are mostly on the run: 21% coming in toward second at 5 m/s or more, 35% across at 5 m/s or more, 32% going away (balls past him or to the wall).

## 2. Long throws (60 m and up, every outfield throw)

| Throw length | n | To home | Comes down short of the target | Median short by | Lateral miss: median, > 3 m | Mean elevation | Eased (effort < 1) |
|---|---:|---:|---:|---:|---|---:|---:|
| 60–70 m | 1,999 | 1 | 17% | 1.5 m | 0.83 m, 2.6% | 14.3° | 55% |
| 70–80 m | 2,103 | 9 | 24% | 1.7 m | 0.98 m, 3.9% | 17.7° | 39% |
| 80–90 m | 2,936 | 2 | 28% | 2.1 m | 1.20 m, 7.4% | 15.1° | 7% |

- "Comes down short" is the path reaching the ground before the target's distance: the 3% speed noise near a solved aim. 3–7% of the 70–90 m throws are past the arm's reach on the fly (thrown at 35°).
- The trace measured no throw that arrived "wide" in a way that matters: none missed by 6 m. Where the throws come down short, they are within 2 m of the receiver.
- At these lengths a throw aimed at chest height at the base peaks 5–9 m up near the cutoff man (§4's table). A cutoff man cannot take it.

## 3. Sources

**The gain from momentum toward the target (the crow hop):**

| Source | What | Tag |
|---|---|---|
| Wagner, Pfusterschmied, von Duvillard & Müller 2011, *J Sports Sci Med* 10:73, Table 1 | 14 elite team-handball players. Standing throw without run-up 22.3 m/s, with run-up 23.9 m/s (93% against 100%). Body (centre of mass) speed toward the goal 1.5 against 3.0 m/s. Mean radial error 0.30 m both; misses 16% against 20% (not significant). "In javelin throwing, release velocity can be considered as the sum of run-up velocity and velocity generated by the thrower" (Bartlett & Best 1988, cited) | `read` |
| Bayios & Boudolos 1998, quoted in Wagner 2011 | Greek elite handball: 23.5 m/s without run-up, 26.3 with | `read` (secondary) |
| Fleisig et al. 2016, *Sports Health* 9:210, Table 1 | 25 pitchers, 5-oz ball, maximal effort: pitch from a mound 34.2 m/s, crow-hop throw on flat ground 37.7 m/s (+10%). "Increased ball velocity … with crow hop throws, likely because of running forward while throwing" | `read` |
| Bouagina et al. 2021, *Biol Sport*, control group baseline | Junior handball: standing throw 21.85 m/s, three-step run-up 22.78 m/s | `read` (excerpt) |
| Wong, Laudner, Evans & Meister 2021, letter on Lizzio et al. 2020 (*Am J Sports Med*) | Crow-hop throws 3.2, 3.2, 3.8 and 2.6 mph faster than standing throws at 30, 45, 60 and 90 ft (interval-throwing effort, not maximal) | `snippet` |
| MLB.com Cut4, 2015, the fastest outfield throws home | "outfielders have the benefit of a crow hop — so they're throwing a bit faster than they could from a mound" | `read` (excerpt) |
| MLB.com, Statcast's best defensive plays of 2016 | Hicks's 105.5 mph throw, the fastest of 2015–16: "setting himself up to catch a fly ball in deep left as he moved toward the plate, then crow hopping and firing" | `read` (excerpt) |

So a throw with the body moving toward the target at about 3 m/s is about 7% faster than one from a stand, about 1.07 m/s of ball per m/s of body speed, at the same accuracy. Statcast's arm figure (top 10% of a player's throws) is of throws with a crow hop.

**The plant and catch-to-release on a fly:**

- Braking limit: 4.75 m/s², the mean of measured maximal decelerations of team-sport athletes (Harper et al. 2020; agent-models §5.3, `read` there).
- No measurement was found of how long a fielder takes to stop, plant and throw after a catch on the move, nor of an outfielder's catch-to-release on a fly in a race. `unsourced`.
- Petriello (MLB.com, 2017), Reddick's throw on Taylor: caught 276 ft out "just above his shoulders" and "came up throwing"; 91.4 mph; "It took Reddick 3.82 seconds to get the ball from his glove to his throwing hand to Brian McCann", the throw pulling the catcher 15 ft up the line. `read` (excerpt). By the sim's throw table, 91.4 mph (40.9 m/s) over the ~80 m to where it was taken is about 2.65 s of flight, so his release was about 1.1 s after the catch: slower than the sim's 0.85 s. One play.
- Tango (2020, Statcast Lab: batter-runner against outfielder at second): outfielder's pickup-to-release plus the infielder's catch-to-tag, "relying on empirical data … 1.2 seconds is reasonable". `read`. The sim's is 0.85 s + 0.25 s ≈ 1.1 s.
- Coaching texts (Marietta Baseball Academy, 2026; BattingLeadoff, 2026): set up "one or two steps behind where the ball will land", catch moving toward the target, crow hop, throw through the cutoff man's head; "a low line that reaches the plate on one long hop". `read`; coaching, not measurement.

**Throws through the cutoff and to the relay man:**

- No MLB count of throws cut, let through or relayed by depth was found. `unsourced`. Statcast publishes no such stat.
- SMT Data Challenge 2025 (Rajaram, Kalsched, Frederick & Zaletski; minor-league tracking data): of 799 outfield throws on base hits with runners on first or second, the cutoff man cut and held 530, let it continue 177 and cut and relayed 92. They assume every outfield throw was cuttable. `read`.
- Kagan, "The Physics of the Cutoff" (THT 2014): an outfielder 270 ft out, a cutoff man 90 ft from home, 90 mph and 2,000 rpm, with air: direct 2.75 s at 11.6°; two throws 1.65 + 0.74 = 2.39 s plus the cutoff man's catch-and-throw. "The peak is near the location of the cutoff. Perhaps this explains the standard practice of making sure the throw from the outfield bounces before reaching home." Part II (2016): the best split of a 400 ft relay between equal arms is 200/200 ft. `read`.
- The owner's write-up (docs/baseball-throw.md): relay man "almost every time" from 280+ ft; through the cutoff on singles and sacrifice flies.

## 4. The cuttable throw against the relay, by the sim's physics

**Revised 2026-11-20 (Opus catch-throw review P2-1, P3-2).** The first version of this section measured the cuttable throw past the cutoff man at the relay spot, 45 m from the thrower. That throw lands 13–35 m short of the plate and reaches it rolling after two to four bounces, which is not the owner's throw: "low, at his head or chest, so the ball carries home on the fly or one hop" (docs/baseball-throw.md). Its table is kept at the end of this section as the record of the first build.

A throwaway probe (`zz_lowthrow2.rs`) flies the throws home from straight-away centre with the sim's throw and bounce physics (drag, 1,500 rpm backspin, grass and dirt):
- the plain aim (chest height at the plate);
- the throw through a cutoff man 14 m (about 45 ft) short of the plate (`through_spot`), passing him no higher than 2.0 m; it lands 7–10 m short and reaches the plate on one hop;
- the same throw past a deep ball's relay man at the relay spot (`cut_spot`), which rolls in;
- the relay: the outfielder's throw to the relay man at the relay spot, the sourced 0.75 s exchange, his throw at 37 m/s.

Times are from release to the ball reaching the plate, at any height. The miss cost (each throw's expected run, from a stand, to whatever of its miss is beyond 1.5 m of reach, at σ 1.2°) is added in the last column for each route (direct / relay).

| Throw | At 37 m/s: plain / through / past the relay man / relay | At 40 m/s: plain / through / past / relay | Miss cost: direct / relay |
|---|---|---|---|
| 60 m (197 ft) | 2.00 / 2.02 / 2.02 / 2.59 s | 1.83 / 1.84 / 1.84 / 2.47 s | 0.09 / 0.03 s |
| 70 m (230 ft) | 2.45 / 2.48 / 2.56 / 2.89 | 2.23 / 2.25 / 2.30 / 2.78 | 0.13 / 0.03 |
| 80 m (262 ft) | 2.97 / 3.02 / 3.25 / 3.22 | 2.68 / 2.72 / 2.89 / 3.11 | 0.17 / 0.04 |
| 85 m (279 ft) | 3.27 / 3.33 / 3.51 / 3.39 | 2.92 / 2.97 / 3.21 / 3.28 | 0.19 / 0.05 |
| 90 m (295 ft) | 3.64 / 3.67 / 3.87 / 3.57 | 3.19 / 3.25 / 3.45 / 3.46 | 0.21 / 0.07 |
| 95 m (312 ft) | 4.13 / 4.01 / 4.20 / 3.76 | 3.49 / 3.55 / 3.77 / 3.64 | 0.23 / 0.08 |

- **The throw through costs 0.02–0.06 s over the plain aim.** Through a man 45 ft out it carries home on one hop, as the owner describes.
- **Crossover, the throw through against the relay, with the miss cost:** about 75 m (246 ft) at 34 m/s, 84 m (276 ft) at 37 m/s and 92 m (302 ft) at 40 m/s. By time alone it is about 3–5 m deeper. The owner's "about 280 ft" lies among them. (The first version's miss cost ran the mean excess and overstated each throw's cost about twice; review P3-2.)
- **In the game the man who goes out decides it first.** `relay_job` sends a relay man out on a ball more than 80 m (262 ft) from home; under that, the cutoff man stands 45 ft out and the throw goes through him. Past a relay man the low throw rolls in (column 3), so the relay wins there. The depth at which the relay man goes out (`RELAY_DEEP_M`, agent-models §5.7) is positioning, not this job; it is about 15 ft shallower than the crossover at a grade-50 arm.
- The plain rainbow is quicker than both, which is why a time-only choice threw it from everywhere (relay review P2-2).

*The first build's table* (`zz_lowthrow.rs`; the cutoff man at the relay spot; plain / past him / relay):

| Throw | At 37 m/s | At 40 m/s | Plain throw's height at that man |
|---|---|---|---|
| 60 m (197 ft) | 2.00 / 2.02 / 2.59 s | 1.84 / 1.84 / 2.47 s | 3.5–4.1 m |
| 70 m (230 ft) | 2.45 / 2.56 / 2.89 | 2.24 / 2.30 / 2.78 | 5.3–6.4 |
| 80 m (262 ft) | 2.97 / 3.25 / 3.22 | 2.68 / 2.89 / 3.11 | 7.5–9.4 |
| 85 m (279 ft) | 3.28 / 3.51 / 3.39 | 2.93 / 3.21 / 3.28 | 8.8–11.3 |
| 90 m (295 ft) | 3.64 / 3.87 / 3.57 | 3.20 / 3.45 / 3.46 | 10.3–13.7 |
| 95 m (312 ft) | 4.13 / 4.20 / 3.76 | 3.50 / 3.77 / 3.64 | 12.3–17.8 |

## 5. After the change (`work/catch-throw`, against main `ab5c374`)

**The final build** (`797fa6f`: the owner's Q33, with the Opus review's and re-review's fixes):
- The plant charges only motion away from the target.
- A throw through a cutoff man 45 ft out passes his spot at his head and carries to the base on one hop. It is kept low where he is going, not where he stands at the release. A deep ball's relay man stands at the relay spot.
- The holder chooses the route by time and miss cost. Runners believe a cuttable throw only where a man is visibly in it.
- A runner names the base a throw in the air is going to from its direction. He follows it on through the bounce only when a cutoff man is visibly in it.
- The momentum gain and the receivers' read of a hop are in. The set-up behind a fly is not.

The same probe, with the trace, on direct seeds 0–299 and 1000–1299. Both seed sets are pooled unless split as [seed 0 / seed 1000]. Two earlier builds are kept as the record behind the reviews and §6:
- "after the first review" (`55402ee`);
- "Q33, before the review" (`4f18c47`).

**What earlier builds did wrong** (all fixed):

- **Returns to second thrown low.** A cutoff man lined up toward home was taken as standing in returns to second, so 98–100% of 60–90 m returns were thrown low and came down 12–18 m short. Fixed: only a man with the job for that base, within 5 m of its line (`CUT_LINE_M`).
- **The set-up behind a fly dropped catches** (below), and was taken out.
- **The low throw rolled in** (review P2-1). It passed the cutoff man at the relay spot, landed 13–35 m short of the plate and rolled in. Then (re-review P3-3) it was kept low where he stood at the release, often 18–25 m out on his way in, and still rolled in. Now he goes to 45 ft out and the throw is kept low at that spot (§4). In the final build 7 of the 8 throws through a cutoff man in 600 games reached the plate on the fly or one hop (1 on the fly, 6 on one hop); 1 took two bounces.
- **Runners believed a throw that was not coming** (review P1-1). Every runner believed a cuttable throw on every outfield throw to third or home from 50 m out, where the world threw it 7 times in 190. That, not the plant, raised tags from second 14% → 21% on the Q33 build. Now only where a man is visibly in the throw.
- **The runners' read of a low throw made them too bold** (review P3-1, re-review P2-1, P3-1).
  - The first versions followed a throw on when it came down 3 m, then 8 m, from every bag, and named the first bag it passed within 2.5 m. A noisy read of a throw to second that passed the bag a couple of metres wide or high was named home, first or third.
  - That made runners bolder on ordinary throws: +0.07 triples and +0.05 outfield assists a team-game on both seed sets.
  - The throw it was written for was never followed: the cutoff man stood within 4 m of where it came down. So runners read it at the plate 0.5–0.75 s early.
  - Now the base is the one on the throw's line nearest where it comes down. He follows it on only when a man with the cutoff job for that base visibly stands on its line.
  - In a throwaway probe, 1,000 noisy reads 0.3 s after release:
    - plain throws to second from centre, left-centre and right-centre, and to third and home, read exactly as the old read did (wrong base 18–100 per 1,000);
    - the throw home through a cutoff man is read 0.07–0.10 s before it reaches the plate, against 0.48–0.69 s before.
  - Triples and assists came down to the read-off ablation's level (below).

**Release after the catch:**

| | Main | Final build | Q33, before the review | As decided (Q32) |
|---|---:|---:|---:|---:|
| Outfield pickups: mean pickup-to-release (first throw) | 1.007 / 1.005 s | 1.081 / 1.079 s | 1.081 / 1.078 s | 1.459 / 1.456 s |
| the same, over 1.2 s | 6.0% / 6.1% | 23.0% / 22.7% | 23.0% / 22.7% | 60.9% / 60.8% |

Tango's empirical pickup-to-release plus the infielder's tag is about 1.2 s (§3), so about 0.95 s of release. This build is about 0.13 s slower than that, main about 0.05 s.

**Balls run down going away** (review P2-2; outfield pickups with the motion mainly away from the throw's target, all throws to second, 600 games):

| Away at the pickup | n (final) | Plant | Pickup-to-release: main → final |
|---|---:|---:|---|
| 3–5 m/s | 1,695 | 0.35 s | 1.11 → 1.26 s |
| 5–7 m/s | 494 | 0.80 s | 1.09 → 1.69 s |
| 7+ m/s | 34 | 1.12 s | 1.11 → 1.99 s |

A real outfielder slows into a ball he runs down and throws off the deceleration. The sim's reaches it at full speed and then owes the whole stop. That is the movement gap Q33 deferred the across term for, and it applies to away motion too. It is the next job's (the approach); Q33 stands until then. `PLANT_OVERLAP_S` is the plant's calibration knob (agent-models §5.5), to be re-checked there against Tango's 1.2 s and the triple rate.

**The set-up behind a fly was taken out of the build.**
- It seldom triggered. The fielder needs, by his own read, time to reach a spot 2.5 m behind the catch and half a second more, and the sim's outfielders rarely have that (§1): about one come-through in two games.
- At the 4 m/s pace that keeps the catch in reach it adds at most about 1 m/s of throw.
- It dropped catches. Coming through at full speed: fly-ball hits .097 → .145 and pop-ups .006 → .028 (100 games). At 4 m/s over 300 games (seed 0): fly-ball hits .097 without it and .104 with it, pop-up hits .004 against .016. In a traced case (seed 6, play 40, right fielder) he drifted back off his catch point coming through and the ball fell in front of him.
- It belongs with the approach work of §6. agent-models §5.5 keeps the rule as specified, marked not built.

**Routes** (outfield throws to third or home, the first throw of the play, 600 games, final build):

| Throw length | To third: relay / low through the cutoff man / plain | To home: relay / low through / plain | Through the cutoff man, at the base on the fly or one hop |
|---|---|---|---|
| under 60 m | 0 / 0 / 42 | — | — |
| 60–75 m (197–246 ft) | 1 / 0 / 108 | 0 / 6 / 3 | 7 of 8 throws through a cutoff man, all home (any throw of the play, both ranges): 1 on the fly, 6 on one hop, 1 on two |
| 75–85 m (246–279 ft) | 1 / 0 / 2 | 2 / 2 / 1 | |
| 85–95 m (279–312 ft) | 0 / 0 / 1 | 2 / 0 / 0 | |

- **Throws to third are almost all plain.** With a runner on second the relay job goes toward home (`relay_job`), and nobody goes out for a ball under 50 m from home.
- **Where a man is in a throw home, the route follows §4.** Through a cutoff man 45 ft out at 60–85 m, to the relay man from deep. "Plain" throws home are ones where he was not yet near the line, or not yet known to the holder.
- **Relays:** the probe's count (an infielder taking an outfielder's throw away from the bags and throwing on) went from 9 to 19 per 600 games.
- **The relay man goes out at a fixed 80 m** (`RELAY_DEEP_M`), whatever the thrower's arm. For a strong arm at 269–295 ft that relays where a throw through the cutoff man would be 0.1–0.2 s quicker (re-review P3-2). It is §5.7 positioning, deferred.

**The game measures** (MLB where known):

| Measure | Main | Final build | After the first review | Q33, before the review | MLB |
|---|---:|---:|---:|---:|---:|
| Runs per team-game (probe; seeds 0 / 1000) | 4.24 / 4.36 | 4.27 / 4.43 | 4.28 / 4.42 | 4.28 / 4.47 | 4.4–4.5 |
| Runner on third, outfield catch, 0–1 out: scored | 62.1% (256 of 412) | 63.9% (274 of 429) | 61.3% | 63.4% | 75–80% |
| the same, put out going | 1 | 3 | 4 | 3 | 2–3.5% |
| Runner on second, third empty, outfield catch: advanced to third | 14.2% (92 of 647) | 15.2% (96 of 631) | 15.2% | 21.2% | 30.2% |
| Singles: first to third | 31.9% (689 of 2,162) | 33.4% (731 of 2,189) | 34.0% | 33.9% | 31.7% |
| Singles: second to home | 54.7% (568 of 1,038) | 53.6% | 54.7% | 55.4% | 61.7% |
| Doubles: first to home | 77.7% (424 of 546) | 79.6% (397 of 499) | 78.6% | 78.9% | 38% |
| Outfield assists per team-game (the probe's generous count) | 0.252 | 0.287 | 0.336 | 0.292 | 0.133 |
| the same, at home (600 games) | 4 | 10 | 9 | 7 | — |
| Triples per team-game | 0.89 / 0.84 | 1.02 / 1.02 | 1.10 / 1.09 | 1.04 / 1.02 | about 0.14 |
| Doubles per team-game | 1.84 / 2.06 | 1.73 / 1.86 | 1.68 / 1.80 | 1.70 / 1.87 | — |

**Tag-ups by depth** (whether or not the runner was on the bag at the catch; runs scored or bases taken / chances, both sets):

| Catch depth | From third: main | From third: final | From second: main | From second: final |
|---|---:|---:|---:|---:|
| under 225 ft | 1 / 27 | 1 / 31 | 0 / 41 | 0 / 38 |
| 225–250 | 0 / 25 | 0 / 30 | 0 / 59 | 0 / 56 |
| 250–275 | 1 / 59 | 4 / 59 | 1 / 84 | 0 / 88 |
| 275–300 | 22 / 56 | 39 / 68 | 1 / 110 | 1 / 100 |
| 300–350 | 157 / 168 | 155 / 165 | 45 / 236 | 45 / 228 |
| 350+ | 75 / 77 | 75 / 76 | 45 / 117 | 50 / 121 |

**Every tagging runner sent** (the throw-race note's counterfactual, `BB_TAG_GO`; runners on the bag at the catch). This was measured after the first review (`55402ee`) and not re-run for the re-review fixes: those change only throws through a cutoff man and runners' reads of them, and these are tag-up throws home. Main's figures are the throw-race note's §8, from its own probe on the same seeds. Bins of 15–60 runners are good to about ±10 points.

| From third, catch depth | Main: safe | After the first review: safe (out) | Q33, before the review | MLB, at least |
|---|---:|---:|---:|---:|
| 225–250 ft | 6 of 13 (46%) | 7 of 15 (47%; 8 out) | 63% | — |
| 250–275 | 17 of 35 (49%) | 23 of 41 (56%; 14 out) | 58% | 42% (5 in 6 of those sent) |
| 275–300 | 42 of 46 (91%) | 47 of 55 (85%; 2 out) | 86% | 86% |

From second, every tagging runner sent: safe 33% at 250–275 ft and 42% at 275–300 ft (main, the throw-race note's §8: 42% and 46%).

**What moved and why** (final build):

- **Runs barely move:** 4.24 → 4.27 [4.47 → 4.57] in the standard runs. Seed 1000's +0.10 is inside one set's noise (about 0.12). In the probe's count it is 4.36 → 4.43.
- **Tags from second are at main's level** (14.2% → 15.2%, MLB 30%). The Q33 build's 21% came from the runners' false cut belief (review P1-1). The plant does not move them.
- **Scoring from third is about unchanged** (62% → 64%, MLB 75–80%). By depth it is up at 275–300 ft (22 of 56 → 39 of 68), within each bin's noise elsewhere.
- **Triples rise** 0.87 → 1.02 a team-game (MLB about 0.14), and **first to third on singles** 31.9% → 33.4% (MLB 31.7%). That is the plant on balls run down going away: with the old flat footwork, the Q33 build's triples were main's (review P1-1's table).
  - The +0.07 that the runners' first read of a low throw added (re-review P2-1) is gone: 1.02 / 1.02 against the read-off ablation's 1.03 / 1.02.
- **Outfield assists rise** 0.25 → 0.29 a team-game (MLB 0.133 by Retrosheet's stricter count). They are at the read-off ablation's level (0.255 / 0.320 against 0.250 / 0.320).
  - On seed 1000 the rise over main is split among the plant, the throw through the cutoff man and the runners' cut belief, within about 2σ of one another. The first review's ablations there: Q33 0.328, old footwork 0.312, no cut belief 0.303; main 0.263.
  - It is not the plant alone, as the ledger first said.
- **First to home on doubles did not move** (77.7% → 79.6%, MLB 38%). It is not the throw; it stays its own problem (TASKS).

## 6. The owner question (Q33) and the answer

**The plant across the line, with fielders who never slow into the ball.**

- *What the game did (the build as decided in Q32, `4ebf724`–`7926848`).* A fielder who takes the ball moving away from his target, or across the line to it, must stop that motion under the braking limit before a set throw, or throw off balance. An outfielder who picks up a ball in the gap running across at 7 m/s needs about a second more than before to be set; one running away from the infield the same.
- *What is wrong.* The sim's fielders reach a ball they are racing at full speed. They never slow into a pickup, or round the ball to take it moving toward their throw, as real fielders do. So they pay the whole stop on plays where real fielders pay little (§5):
  - an outfielder's pickup-to-release rose from 1.01 s to 1.46 s on average, and 61% now take over 1.2 s. Tango's empirical figure for pickup-to-release plus the infielder's tag is about 1.2 s (§3, read): main matched it, this build does not;
  - runs 4.24 → 4.69 [4.47 → 4.94] in the standard runs, triples +35%, ground-ball hits .313 → .327;
  - three frozen infield regression plays now fail (`bb_game/tests/throw_choices.rs`: a lead force on seeds 13 and 23, the 6-4-3 on seed 3): the infielder fields moving across and loses the time. With option 3 all five pass; with option 2, four.
- *Options:*
  1. Keep it as built. The game becomes faster on the bases than MLB in most measures.
  2. Count the motion across the line at the fielder's cornering limit (`Agility`, 7 m/s² at grade 50), and only the motion away at the braking limit: seed 0 gives 4.45 runs, ground-ball hits .318. A physical reading (he curves toward his target rather than stopping), but wrong for a man running straight across.
  3. Count only the motion away, at the braking limit: 4.12 at 100 games on seed 0, against 4.09 for the old flat rule.
  4. **(Recommended)** Keep the physics and build the approach: a fielder who will throw after fielding the ball slows into it and, with room, rounds it, so that he takes it moving toward his target (agent-models §5.2's arriving under control, extended to ground balls and caroms), with the set-up behind a fly in the same job. That is how real fielders avoid the cost, and Tango's 1.2 s is the check.
- *Recommendation:* option 4 as the next job. Until it is built, option 2 or 3 on this branch, so that main does not carry half a run from a movement gap. The switch is one function (`throwing::footwork_s`) and its tests; nothing else in the build depends on it.

**Owner's answer (Q33, 2026-11-19): option 3 on this branch for now.** Charge only motion away from the target, at the braking limit; motion across the line is not charged until the approach is built. The approach (slowing into and rounding the ball so that the fielder takes it moving toward his target, with the set-up behind a fly) is the next job. Built in `4f18c47` (`throwing::footwork_s`, `away_mps`); agent-models §5.5 records the across term as decided in Q32 and deferred by Q33. §5 below is re-measured on that build.

Two findings are reported, not decided here:

- **Outfielders creep in to routine flies** (§1). Whether they should camp under every routine fly is the movement model's (agent-models §5.2). It also decides how often the set-up behind a fly could happen, which is why that waits for the approach work.
- **The cut call** (the catcher's "Cut!") stays out of this job, as the job says. The sim's cutoff man never intercepts a throw aimed past him, so a low throw he could have cut goes on to the base: the "let it go" half of the call. The route change does not need the call to be sensible, since throws through the cutoff are few (§5).

## Sources

- Wagner H, Pfusterschmied J, von Duvillard SP, Müller E (2011), Performance and kinematics of various throwing techniques in team-handball, *J Sports Sci Med* 10(1):73–80 (PMC3737895; `read`).
- Fleisig GS, Diffendaffer AZ, Aune KT, Ivey B, Laughlin WA (2016), Biomechanical analysis of weighted-ball exercises for baseball pitchers, *Sports Health* 9(3):210–215 (PMC5435148; `read`).
- Bouagina R et al. (2021), *Biology of Sport*, doi 10.5114/biolsport.2022.106150 (`read`, excerpt).
- Wong R, Laudner KG, Evans DM, Meister K (2021), letter, *Am J Sports Med* 49(8), doi 10.1177/03635465211021409 (`snippet`); on Lizzio VA et al. (2020), The effect of the crow hop on elbow stress during an interval throwing program, *Am J Sports Med* 49(2):359–363 (abstract `read`).
- Harper DJ, Cohen DD, Carling C, Kiely J (2020), *Sports* 8(6):76 (via agent-models §5.3).
- Petriello M (2017), "Statcast: Taylor made right call to stay at 3rd", MLB.com (`read`, excerpt).
- Tango T (2020), "Statcast Lab: Batter-Runner v Outfielder, Play at 2B", tangotiger.com (`read`).
- MLB.com Cut4 (2015), the 10 fastest outfield throws to home; MLB.com (2016), Statcast's best defensive plays of the 2016 season (`read`, excerpts).
- Kagan D (2014, 2016), "The Physics of the Cutoff", Parts I and II, The Hardball Times (`read`).
- Rajaram P, Kalsched J, Frederick A, Zaletski A (2025), "The Man in the Middle", SMT Data Challenge (`read`).
- Coaching: Marietta Baseball Academy (2026), runner on third, fly ball; BattingLeadoff (2026), the do-or-die throw home (`read`; practice, not measurement).
- Probes (throwaway, not tracked): `sim/target/codex-probes/catchthrow/` (`zz_ct_probe.rs`, `zz_lowthrow.rs`, `ct_sum.py`, `catch_sum.py`, `WIP-with-trace.diff`; runs `base_0`, `base_1000`).
