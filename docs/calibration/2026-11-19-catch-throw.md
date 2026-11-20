# The catch sets up the throw: how outfielders take the ball, what their long throws do, and the sources (2026-11-19)

**Job.** The owner's Q32 (agent-models §5.5, "The catch sets up the throw"): the fielder's momentum at the catch, relative to his target, decides the throw's time, speed and accuracy, and outfield throws go low through the cutoff man or to a relay man on deep balls. Step 1 measures the sim as built and finds sources; step 2 is the spec (agent-models §5.5); step 4 measures what the change moves (§5 below). Branch `work/catch-throw`, from main `ab5c374`.

Source tags as in the [throw-race note](2026-11-19-throw-race.md): `read` (page text read), `snippet` (search-engine summary only), `unsourced`.

## Summary

- **Sim outfielders almost never wait under a fly.** Of 8,183 outfield fly catches in 600 games, 17.8% are made standing (under 1 m/s), and only 4.3% by a fielder who had stood still for half a second or more (1.1% for a second). The median speed at the catch is 2.5 m/s. Most catches are made moving across the line to second (45%) or away from it (21%); 16% coming toward it. Real outfielders camp under routine flies; the sim's "arrive under control" rule (agent-models §5.2) keeps them a doubt short of the spot, so they creep in to the catch.
- **The flat footwork rule charges most of these nothing.** On fly catches that led to a throw (5,495), the 0.2 s was charged on 19%: every catch moving away faster than 3 m/s, and a quarter of those moving across at 3 m/s or more (when part of it is away). A fielder caught moving away at 7 m/s owes 0.2 s, against 1.47 s to stop under the sourced braking limit.
- **Long throws are rainbows.** Outfield throws of 60–90 m leave at 14–18° and peak 5–9 m up, over any cutoff man's head. 17–28% come down short of the target (median 1.5–2.1 m short: speed noise near the aim), and the lateral miss is small (median 0.8–1.2 m, more than 3 m on 3–7%). Almost none go home: 12 of 7,038 such throws in 600 games; nearly all are returns to second with no play.
- **Relays have almost vanished** (9 in 600 games), as the relay review found.
- **Sources found:** a run-up toward the target adds about 7% of ball speed (1.6 m/s for 1.5 m/s more body speed, elite handball; read), with no loss of accuracy; a crow hop adds about 10% over a pitch (read). No measurement of an outfielder's plant or catch-to-release on a fly was found, nor an MLB share of throws through the cutoff by depth. One play (Reddick 2017, 276 ft, read) took 3.82 s from catch to the catcher.
- **The physics puts the relay where the owner said.** If a throw with a cutoff man in it must pass him at head height, it comes down short and skips on; by the sim's own flight and bounce physics it is quicker than the relay (with the sourced 0.75 s exchange) up to about 80 m at a grade-50 arm and 90 m at 40 m/s, so the relay wins from about 262–295 ft. The owner's figure is about 280 ft (§4).

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

A throwaway probe (`zz_lowthrow.rs`) flies three throws home from straight-away centre with the sim's throw and bounce physics (drag, 1,500 rpm backspin, grass and dirt): the plain aim (chest height at the plate); the cuttable throw, which passes the cutoff man at his spot (`relay_spot`: the throw's length less 45 m short of the base, at least 14 m) no higher than 2.0 m and then skips on; and the relay (the sourced 0.75 s exchange, his throw at 37 m/s). Times are from release to the ball reaching the plate, at any height.

| Throw | At 37 m/s: plain / cuttable / relay | At 40 m/s: plain / cuttable / relay | Plain throw's height at the cutoff man |
|---|---|---|---|
| 60 m (197 ft) | 2.00 / 2.02 / 2.59 s | 1.84 / 1.84 / 2.47 s | 3.5–4.1 m |
| 70 m (230 ft) | 2.45 / 2.56 / 2.89 | 2.24 / 2.30 / 2.78 | 5.3–6.4 |
| 80 m (262 ft) | 2.97 / 3.25 / 3.22 | 2.68 / 2.89 / 3.11 | 7.5–9.4 |
| 85 m (279 ft) | 3.28 / 3.51 / 3.39 | 2.93 / 3.21 / 3.28 | 8.8–11.3 |
| 90 m (295 ft) | 3.64 / 3.87 / 3.57 | 3.20 / 3.45 / 3.46 | 10.3–13.7 |
| 95 m (312 ft) | 4.13 / 4.20 / 3.76 | 3.50 / 3.77 / 3.64 | 12.3–17.8 |

- The cuttable throw costs nothing up to about 60 m and 0.1–0.3 s at 70–90 m: it comes down short and skips the rest of the way.
- It beats the relay up to about 80 m (262 ft) at 37 m/s and 90 m (295 ft) at 40 m/s. The owner's "about 280 ft" sits between.
- So the rule "low through the cutoff, or the relay, whichever gets there sooner" reproduces the owner's depth from the physics, with the sourced exchange. The plain rainbow is quicker than both, which is why a time-only choice threw it from everywhere (relay review P2-2).

## 4a. Proposed owner questions

None needed for the build: §5.5's decided design covers the plant, the off-balance throw, the crow hop, the set-up and the routes. Two findings are reported, not decided here:

- **Outfielders creep in to routine flies** (§1). The set-up behind the ball (§5.5) changes that only for a fielder with time to spare and a throw to make. Whether outfielders should camp under every routine fly is the movement model's (agent-models §5.2) and is not in this job.
- **The cut call** (the catcher's "Cut!") stays out of this job, as the job says. The sim's cutoff man never intercepts a throw aimed past him, so a low throw that he could have cut goes on to the base. That is the "let it go" half of the call.

## 5. After the change

*(Step 4; filled in after the build.)*

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
