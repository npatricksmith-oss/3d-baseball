# The throw race on tag-ups: what limits sacrifice flies and tags from second (2026-11-19)

**Job.** Tag-ups fall short of MLB. With fewer than two out, runners on third score on 59% of outfield catches (MLB 75–80%). Runners on second tag and take third on 17% / 11% of outfield catches with none / one out (MLB 37% / 26%). Two explanations were on record:
- the ball: every throw leaves at the Statcast top-10% arm speed;
- the runner's read: it becomes sure late on deep flies.

This note splits the cases, takes the race apart and checks each part against a source. It also measures what a realistic spread of outfield arms would do. Branch `work/throw-race`, from main `e40ffd7`. **No sim code was changed.** The job's stop rule applies (§6), so there is no ledger row.

Source tags as in the [tag-up note](2026-11-14-tag-ups.md): `read` (page text read), `snippet` (search-engine summary only; mlb.com refuses automated fetches), `unsourced`.

## Summary

- **From third, the race is the limit, and it is the direct throw home.**
  - Runners on third who tag read the race correctly. Their believed ball (release, full-arm flight, tag) is the world's best case.
  - The world's race, run with every tagging runner sent, still favours the ball by about one depth bin more than MLB's does: about **0.3–0.5 s**.
  - Of the 21-point shortfall in runs scored, about 15 points are this race. About 3 are runners who had not tagged at the catch (the read). The rest is noise and the depth mix.
- **Outfield arm variation is not it.** Outfield arms drawn per game from the 2025 Statcast spread between players moved nothing: 58.8% → 58.2% from third, 13.0% → 12.5% from second (600 games each, within noise). Whatever ratings do, they will not close this gap.
- **The "top-10% arm is too fast" claim cannot be settled from published data, and it is too small to close the gap anyway.**
  - Statcast publishes the mean of a player's top 10% of throws. It does not publish what share of outfield throws are max effort.
  - That share decides whether the top-10% mean sits above or below the mean max-effort throw.
  - The plausible size is a few mph: about 0.1 s of flight on a 250–300 ft throw.
- **Every part that could hold the missing 0.3–0.5 s is unsourced:**
  - the outfielder's catch-to-release on a fly in a race;
  - the share of throws at max effort;
  - the catcher's receive-and-tag;
  - a rolling start for the runner.
- **From second, the read and the race both count.**
  - A quarter of runners on second are not on the bag at the catch. Had they tagged and decided like the others, the advance rate would rise by about 5 points (13% → about 18%).
  - The rest of the gap to MLB's 30% is the race to third.
- **A real, sourced discrepancy that does not limit tag-ups: the relay man.**
  - The sim's cutoff man holds the ball 0.28–0.38 s on throws home. The one measured elite relay exchange is 0.62 s, and an analyst's expectation for a big-league shortstop is 0.75 s (both `snippet`).
  - This is the same fault the double-play pivot had (0.455 s, fixed to 0.80 s in §5.5).
  - It decides most races at 250–300 ft when every runner is sent. It barely moves the tag-up rate, because runners decide against the direct throw.
  - Recommended as its own task (§6).

## 1. The cases (direct seeds 0–299 and 1000–1299, as built)

Outfield catches of the batter with fewer than two out and a runner on third, or a runner on second with third empty. A throwaway trace in the runner's decision (reverted, not committed; `sim/target/codex-probes/throwrace/TRACE-PATCH.txt`) logs every tag-up decision: his believed run time, believed ball time and its parts, his timing draw, the break-even and go/hold. A probe joins it to what happened. Depth is the catch point's distance from home, binned at the sources' boundaries.

**Runner on third** (413 catches; scored 243 = 58.8%: 48.9% with none out, 63.5% with one; put out 5 = 1.2%):

| Catch depth | Not on the bag at the catch | Tagged, held by the race | Sent, safe | Sent, out | MLB (send / safe when sent) |
|---|---:|---:|---:|---:|---|
| under 225 ft | 23 | 5 | 0 | 0 | hold |
| 225–250 | 14 | 11 | 1 | 0 | about 30–40% / 90% |
| 250–275 | 19 | 36 | 1 | 0 | about 50% / 83% |
| 275–300 | 8 | 31 | 15 | 4 | about 90% / 95% |
| 300–350 | 11 (4 scored) | 11 | 145 | 0 | about 100% |
| 350+ | 2 | 0 | 76 | 0 | always |
| all | **77 (19%)** | **94 (23%)** | **238** | **4** | |

MLB references are from the [tag-up note §2](2026-11-14-tag-ups.md#2-sacrifice-fly-outcomes-by-depth-and-the-league-count) (Olszewski 2021, BP 2025, THT 2006; `read`).

**The depth mix is not the cause.** Applying MLB's depth-by-depth scoring rates to the sim's own catch depths gives about 78% scored, against MLB's 75–80%. So the shortfall is in what happens at each depth.

**Runner on second, third empty** (656 catches; advanced 85 = 13.0%: 16.7% with none out, 10.7% with one; none thrown out going):

| Catch depth | Not on the bag | Tagged, held by the race | Sent, safe |
|---|---:|---:|---:|
| under 225 ft | 1 | 40 | 0 |
| 225–250 | 6 | 52 | 0 |
| 250–275 | 26 | 61 | 1 |
| 275–300 | 40 | 68 | 0 |
| 300–350 | 52 | 149 | 39 |
| 350+ | 42 | 37 | 42 |
| all | **167 (25%)** | **407 (62%)** | **82** |

"Not on the bag" is the read: the runner was holding partway when the ball was caught (the [fly-read note](2026-11-19-fly-read.md) §3–§4).

## 2. The runner's belief at the decision

Mean over tag-up decisions at the starting base. All times are seconds after the catch.

| Base, depth | n | His run | Believed ball = release + flight + tag | Margin | Break-even |
|---|---:|---:|---|---:|---:|
| third, 250–275 ft | 37 | 4.10 | 3.75 = 0.85 + 2.65 + 0.25 | −0.35 | 0.56 |
| third, 275–300 | 50 | 4.10 | 4.21 = 0.86 + 3.10 + 0.25 | +0.11 | 0.54 |
| third, 300–350 | 156 | 4.10 | 4.73 = 0.89 + 3.59 + 0.25 | +0.63 | 0.52 |
| second, 275–300 | 68 | 4.10 | 3.46 = 0.86 + 2.35 + 0.25 | −0.64 | 0.82 |
| second, 300–350 | 188 | 4.10 | 4.04 = 0.91 + 2.88 + 0.25 | −0.07 | 0.81 |
| second, 350+ | 79 | 4.10 | 4.53 = 0.91 + 3.37 + 0.25 | +0.43 | 0.81 |

He goes when Φ(margin / 0.20 − 1) beats the break-even (§6.3). That needs a margin of about +0.33 s with none out and +0.12 s with one, from third. From second it needs about +0.30 to +0.42 s. The break-evens are the run table's (Olszewski: 75.1% and 32.5% from third).

**What MLB's send rates require of his belief:**
- At 250–275 ft MLB coaches send about half. From third his margin there would have to be about +0.12 to +0.33 s, which is **0.45–0.7 s** more than now.
- At 275–300 ft they send 90%. That needs about +0.3 to +0.5 s, which is **0.2–0.4 s** more.

**His belief is not the error.** His believed flight is the throw physics at the fielder's full arm (his read of it, σ 1.1 m/s). His believed release is the holder's grade-50 transfer and turn, as observed (§6.4). When every runner is sent (§3), the direct throws home that reached the catcher took longer than he believed: 3.70 s against 3.41 s at 250–275 ft, and 3.53 against 3.20 at 225–250. The shorter the throw, the bigger the difference, which fits the thrower easing the throw when he has time in hand (`EFFORT_MIN` 0.8, §5.5; inferred, not traced). So his belief is the ball's best case. That is what a runner should assume.

## 3. The race in the world: every tagging runner sent

The same trace with a temporary override: a tagging runner always goes from his own base. The defense plays as built.

**Runner on third** (388 catches; 316 tagged):

| Catch depth | Tagged runners sent | Safe | Direct throws home: OF release / ball in the catcher's glove / safe | Relayed: OF release / relay held / ball in glove / safe |
|---|---:|---:|---|---|
| 225–250 ft | 11 | 5 (45%) | 11: 0.85 / 3.53 / 5 | 0 |
| 250–275 | 31 | 7 (23%) | 14: 0.84 / 3.70 / 5 | 15: 0.60 / **0.38** / 3.54 / 1 |
| 275–300 | 45 | 32 (71%) | 3 | 24: 0.52 / **0.28** / 3.83 / 11 (20 not thrown home: 18 safe) |
| 300+ | 225 | 225 | — | — |

The runner reaches home 4.08 s after the catch.

**What MLB implies if every runner were sent:**
- At 275–300 ft MLB sends 90% and 95% of them are safe. So at least 86% of all runners would be safe there. The sim gives 71%.
- At 250–275 ft MLB sends about half and 5 in 6 are safe. So at least 42% would be safe, and more from the runners held. The sim gives 23%.

So the sim's curve sits about 25–35 ft deeper than MLB's. At 30 m/s of throw that is **about 0.3–0.4 s** of ball, the same size as the belief shift in §2. On the old 4.07 s against 4.4 s at 280 ft (agent-models §6.4, "Measured 2026-11-14") it is 0.33 s.

**Runner on second** (603 catches; tagged runners sent: safe 41% at 250–275 ft, 52% at 275–300, 64% at 300–350, 87% at 350+). Most throws to third are direct: 99 at 250–300 ft, 44 safe. The third baseman has the ball at 3.49–3.56 s against the runner's 4.08 s. MLB has no depth table for tags from second. Retrosheet 2024 has 30.2% advancing with 0.6% doubled off ([fly-read note §3](2026-11-19-fly-read.md#3-the-catch-cut-offs)). Outs advancing from second were not counted there.

## 4. The ball's parts against the sources

The direct throw home from a 250–300 ft catch, the race that sets the decision:

| Part | Sim | Source | Tag | Verdict |
|---|---|---|---|---|
| Catch to release | 0.85 s set (outfield `Transfer` grade 50 [E]), plus turn and footwork; measured 0.84–0.88 s on direct throws. A hurried release to a cutoff is 0.49–0.60 s at 82–91% of the arm | Kiermaier's pickup-to-release on a ground ball at the wall, 0.77 s (MLB.com, "Rays' relay even more amazing under dissection"). Catcher exchange, a different job: 0.64 elite, 0.73 average, 0.85 poor (2017). No outfield figure for a fly caught in a race | `snippet`, one play; `unsourced` for flies | Inside anything plausible; nothing to fit to |
| Throw speed | Full effort at the player's top-10% mean (Savant), execution σ 3% | Top-10% definition, and "no rulebook definition of a throw where the player is trying hard … many non-competitive lobs" (MLB.com glossary, Arm Strength). 2016–17 "competitive throws" league averages 86.2 and 88.9 mph, competitive meaning at least 90% of the player's displayed arm. Acuña 2022: 97.9 mph mean on max-effort throws, max 101.5. Savant 2025 CSV: 166 outfielders with ~196 throws (median) each; max − top-10% mean, median 6.1 mph | glossary, 2016–17 articles and Acuña: `snippet`; CSV: `read` | The top-10% mean is above the mean max-effort throw only if more than 10% of an outfielder's throws are max effort. That share is **unsourced**. A 6 mph spread across his top 20 throws fits max-effort throws with σ ≈ 3.3 mph (the sim uses 3%, 2.7 mph). If competitive throws home averaged 1σ below the top-10% mean, flight would grow ~3.5%: **+0.09 s** at 2.6 s |
| Flight | Drag and 1500 rpm backspin, solved elevation; believed 2.56 s at 250–275 ft, true direct throws 2.86 s (eased) | Lang 2014: 280 ft in 2.47–2.57 s at 95–103 mph | `read` (tag-up note §3) | Agrees: about 2.6–2.7 s at 89 mph over the shorter throw |
| Where the catcher takes it | Median 1.4–2.0 m from the plate at 225–275 ft (mean 1.75–3.15); direction σ 1.2° | ±1° of launch ≈ ±5 ft at the plate (Nathan, via the SABR Cespedes article) | `snippet`, not read | Consistent |
| Tag | 0.25 s after securing (`tag_apply_s`) within 1.2 m reach; outs were tagged 0.40–0.44 s after the catcher's catch (the runner arriving) | "a tag at roughly .25 seconds" (MLB.com, Rays relay) | `snippet`, low | Nothing better |
| Runner | 4.08 s from the catch to the plate (league τ 0.76, owner's Q1 target 4.10 s for a standing 90 ft) | 90-ft splits of runners on third in sacrifice-fly situations, mean 4.03 s (Olszewski 2021); Taylor "as quickly as 3.88" | `read`; `snippet` | The sim is ~0.05 s slow; that is the owner's target (Q1). A rolling start's gain is `unsourced` |

The relay route, which wins most of these races when every runner is sent (§3):

| Part | Sim | Source | Tag |
|---|---|---|---|
| Relay man, catch to release | Set at 0.7 × `Transfer` = 0.455 s with no turn ("turns through the reception"). He hurries down to 0.6 of that (0.27 s). Measured: **0.28–0.38 s** on throws home | Tatis Jr.'s "rocket relay" exchange 0.62 s (MLB.com, top defensive plays of 2021); in the Rays analysis, Adames "could be expected to take about 0.75 seconds" | `snippet` (one elite highlight; one analyst's estimate) |
| The runner's belief of a relay | 0.7 × 0.75 = 0.525 s at the relay man (`read_throw_time`) | — | — |

On every route, the sim's relay man is quicker than the one measured elite relay, by 0.25–0.35 s. The double-play pivot had the same 0.455 s with an early release and was set to 0.80 s with no early release, on the same kind of evidence (§5.5). The runner's belief of a relay (0.525 s) is also quicker than the sources, but slower than the truth.

## 5. Outfield arm variation (would ratings close it?)

A throwaway probe (`zz_arm_probe`) gives each game's left, center and right fielders arms drawn about the sim's positional means, with the 2025 spread between players. That spread comes from the Savant arm-strength leaderboard CSV (`baseballsavant.mlb.com/leaderboard/arm-strength?…&csv=true`, `read`, 2025, at least 50 throws at the position, weighted by throws):

| | LF | CF | RF |
|---|---:|---:|---:|
| Mean, mph | 86.1 | 88.3 | 89.2 |
| σ between players, mph | 2.72 | 4.45 | 3.70 |
| Players | 70 | 66 | 76 |

Draws are clamped at ±3σ. Runners read the true arm with their usual σ 1.1 m/s, as built (scouting knows an arm). Both teams share the draw in a game. 600 games:

| | Average arms (as built) | Spread arms |
|---|---:|---:|
| Runner on third, outfield catch, scored | 58.8% (413) | 58.2% (395) |
| Runner on second, advanced to third | 13.0% (656) | 12.5% (654) |

No movement. A spread of arms about the same mean moves the rate only by the curvature of the send-and-safe curve, and around 275–300 ft that curve is nearly straight. **Ratings will make individual races realistic, but not the league rate.**

## 6. Conclusion: stop and report (job step 2)

**What limits the tag-ups:**
- **From third:** the direct-throw race at 225–300 ft, about 15 of the 21 points. The read is about 3 points: runners off the bag at the catch on 300+ ft flies.
- **From second:** the read, about 5 of the 17 points (167 runners not tagged, 94 of them on catches of 300 ft or more), and the race, the rest.

The runner's belief matches the world. The world's race is 0.3–0.5 s quicker for the ball than MLB's outcomes imply.

**Why no fix was built:**
- The measured parts agree with their sources: flight physics, positional arm means, runner speed.
- Every part that could carry 0.3–0.5 s is unsourced, or sourced only by single highlights:
  - the outfielder's catch-to-release on a fly in a race;
  - the share of throws at max effort, and so where the true mean throw speed sits below the top-10% figure (worth about 0.1 s at most on present evidence);
  - the catcher's receive and tag;
  - a rolling tag-up start.
- Choosing which to lengthen is a parameter with no source, which the job reserves for the owner.

**Proposed owner question** (the Coordinator numbers it):
- *What the game does now.* On a fly caught 250–300 ft out with a runner on third, the outfielder's throw home (0.85 s to let go, full arm, a 0.25 s tag) beats an average runner (4.1 s) or comes out even. So runners hold on about two of three such flies. Big-league runners go on about half at 250–275 ft and on 90% at 275–300 ft, and score 83–95% of the time.
- *What is wrong.* Somewhere the real play is 0.3–0.5 s slower for the ball, and no published measurement says where.
- *Options:*
  1. **(Recommended) Wait for measurements.** Get Statcast's tag-up times from the catch ("3rd to Home", defined in 2025) and any throw-level speed data; the Savant CSV endpoints can be fetched. Leave the race as it is; build the relay fix below.
  2. Lengthen the outfielder's catch-to-release on a throw home after a fly by a named [E] amount (about 0.3 s, to ~1.15 s), unsourced.
  3. Give each throw a max-effort speed drawn below the top-10% figure (about one σ, ~3 mph), partly grounded. It closes only about 0.1 s.
  4. Give a tagging runner a rolling start (`tau_gain` below 1), unsourced. The tag-up note advised against it without a measurement.
- *Recommendation:* option 1. Options 2–4 each fit a rate with a guessed number.

**A separate sourced fix, not built here (scope):** the relay man's catch-to-release.
- Model it on the pivot: about 0.75 s at a `Transfer` grade of 50, scaled by the grade rule (0.64 s at grade 70, near the measured elite 0.62 s), with no early release.
- The runners' belief of a relay (`read_throw_time`, `typical_throw_time`) and the holder's own estimate of a relay route (`throw_race`) should follow, since they are league knowledge.
- Expected effect:
  - When every runner is sent, relayed throws home at 250–300 ft (39 of 56 throws to the catcher) would arrive about 0.35 s later. The direct throw then sets those races.
  - Outs on runners sent from third at 275–300 ft fall (4 of 19 now, MLB about 5%).
  - The tag-up rate barely moves.
  - It moves every relay in the game (extra-base hits, second to home on singles), so it needs its own ledger row and review.

## 7. Open

- **The held runner never goes later.** A runner standing on third after deciding to hold never decides again while the ball is held: a runner on a base takes no relook (`runner_relook` returns early). A real runner goes when the outfielder throws to second. It did not show in these cases. Not measured.
- **Tags from second have no MLB depth table and no thrown-out rate** here. A Retrosheet count of outs on runners advancing from second on outfield catches would give the latter.
- **Statcast hit distance and the sim's catch point** may differ by a few feet (catch height on the descent, about 5 ft). They were compared directly.

## Sources

- Baseball Savant arm-strength leaderboard CSV, 2025 (`read`, downloaded 2026-11-19).
- MLB.com glossary, Arm Strength (`snippet`).
- MLB.com 2016–17 Statcast arm articles: "competitive throws" 86.2 and 88.9 mph, the 90%-of-displayed-arm definition (`snippet`; the article giving the 90% wording was not identified).
- MLB.com, Acuña owns 2022's strongest throwing arm (`snippet`).
- MLB.com, Rays' relay even more amazing under dissection: outfield release 0.77 s, the shortstop's expected 0.75 s, the 0.25 s tag (`snippet`).
- MLB.com, top defensive plays of 2021: Tatis Jr.'s 0.62 s relay exchange (`snippet`).
- MLB.com, catcher exchange 2017 (`snippet`).
- From the tag-up note (`read` there): Olszewski 2021, BP/Dubuque 2025, THT/Fox 2006, THT/Lang 2014.
- Probe outputs and scripts (throwaway, not tracked): `sim/target/codex-probes/throwrace/` (`zz_race_probe.rs`, `zz_arm_probe.rs`, `summarize.py`, `TRACE-PATCH.txt`, runs `base_*`, `go2_*`, `arm_*`).
