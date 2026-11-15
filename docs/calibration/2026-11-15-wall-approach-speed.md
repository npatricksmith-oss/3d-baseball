# How fast do outfielders reach the wall? (research note)

Status: research only, no sim change. Question from the owner: before choosing a wall-contact speed for `move_fielders` (today: speed toward the nearest wall capped at `sqrt(2 * BRAKE * dist)`, BRAKE = 6 m/s², so arrival is near 0), find real data.

**Short answer: the data that would settle it (a fielder's measured speed at the instant of wall contact) is not public.** What exists is indirect. Every number below is marked SOURCED or ESTIMATE. Nothing here was measured by us.

## 1. Arrival speed at the wall in MLB

What I found and did not find:

- **No published distribution of arrival speed.** I found no peer-reviewed study, MLB release or team report that measures outfielder speed at wall contact. Statcast tracks fielders (it has Sprint Speed and the fielder-path data behind Catch Probability) but, as far as I could find, publishes no "speed at wall" field.
- **No biomechanics or injury study of outfielder-wall collisions** turned up. Searches for impact velocity, padding test standards and MLB warning-track standards returned vendor guides (padding thickness 3 in for amateur, 4 in for pro, no test data cited), a patent (US 5,795,252, outfield wall structure, which says fielders slow when they reach the track), and news items. The one nearby academic paper (Strickland & Bevill, J. Appl. Biomech. 2019, softball fielder's masks vs. batted balls) is about balls, not walls.
- **Peak running speeds on wall plays (SOURCED, MLB.com Statcast posts, found via search snippets; dates and exact URLs not captured, so treat as unverified until re-opened):** Brett Gardner, max 17.2 mph (7.7 m/s) tracking a fly at the wall; Mike Trout, max 17.1 mph (7.6 m/s) over 77 ft, then into the padded wall; Jacoby Ellsbury, max 19.9 mph (8.9 m/s) on a line drive to left-center. These are **top speeds during the run, not arrival speeds**; each says nothing about speed at contact. They do show full-speed running up to the last stretch is common on the highlight plays.
- **Average speed over a wall run (SOURCED, MLB.com, "Catch Probability updated to account for walls", 2018, https://www.mlb.com/news/catch-probability-updated-to-account-for-walls-c269814542):** Stanton ran 96 ft in 5.4 s into the Colorado wall. That is 17.8 ft/s = 5.4 m/s mean, consistent with accelerating from rest. Pillar ran 110 ft into the fence in the same article. No arrival speed given.
- **Collision injuries persist despite the track** (SOURCED, news only: ESPN "baseball warning track useless", https://africa.espn.com/mlb/story/_/id/8313341/baseball-warning-track-useless; Twins concussions leading to new padding at Target Field, San Francisco Chronicle / Uproxx). So some fielders do arrive fast enough to be hurt. No speeds.
- **Frame-by-frame analyses of famous wall catches:** I found none with measured speeds. A broadcast-video or Statcast video analysis could produce a few data points by hand; that is a possible follow-up, but it would be a handful of highlight plays, a biased sample.

## 2. What fielders do near the wall

All qualitative; no counts found.

- The warning track exists to tell a fielder looking at the ball that the wall is near; fielders "often slow down when they reach it" (patent US 5,795,252). Typical width about 15 ft (Wikipedia "Warning track"; varies by park, ESTIMATE as a round figure for the sim).
- ESPN column (link above): fielders keep eyes on the ball and build their own mental warning track, easing off before reaching the dirt. A Substack piece (leejudge.substack.com, "how to spot a bad outfielder") says weaker fielders drift and slow once on the track. Opinion, not measurement.
- Leaping, bracing with a hand or foot: no frequency data found. Statcast's wall-ball definition caps counted height at 8 ft, which implies leaps and climbs are common enough to matter, nothing more.
- Tom Tango's abandon-rate study (tangotiger.net, Abandon Rate Model Blog.pdf, found via search, not read in full) compares wall/back plays with others; I did not extract numbers. Worth reading if the sim later models giving up on a ball.

## 3. How hard can an athlete decelerate? Is 6 m/s² in range?

SOURCED, Harper, Cohen, Carling, Kiely, *Sports* 2020, 8(6), 76 (https://clok.uclan.ac.uk/id/eprint/33478/1/sports-08-00076.pdf; I read the full text). 27 male university team-sport athletes, 20 m sprint then maximal stop, radar-measured. **HDEC is the mean deceleration from peak speed to the lowest speed reached.**

| Group | Approach speed | Mean deceleration |
|---|---|---|
| high-HDEC (n = 13) | 7.80 ± 0.44 m/s | 4.99 ± 0.24 m/s² |
| low-HDEC (n = 14) | 7.37 ± 0.28 m/s | 4.24 ± 0.41 m/s² |

Further, from search snippets of related Harper work (unverified, not opened): about 4.45 ± 0.61 m/s² mean after a 20 m sprint (university athletes); about 3.9 m/s² after a 10 m sprint (youth); a conference slide listing peak instantaneous 10.25 m/s² against mean 5.28 m/s² (cohort unclear, do not rely on it). Deceleration ability depends on approach speed (Philipp et al., cited in Chen et al.), so quote speed with any figure.

Reading for the sim: a trained team-sport athlete, braking on purpose from about 7.5 m/s with no ball to track, averages about 4.2 to 5.0 m/s² (SOURCED). The sim's constant 6 m/s² is **above the measured mean by roughly 20 to 40 %**, though below the instantaneous peaks (which are higher than the mean; the single 10 m/s² figure is unverified). For an outfielder who must also catch a ball, turn, and look at the wall, a sustained 6 m/s² is optimistic. ESTIMATE: 4.5 to 5 m/s² would match the measured mean. Caveat: the test stops on a flat indoor or track surface with the athlete's full attention; grass and cleats differ and no baseball-specific figure exists.

Direction of the effect: lower BRAKE makes the fielder start slowing earlier (stopping distance v²/2a: from 8 m/s, 5.3 m at 6 m/s², 6.4 m at 5 m/s²), so it would cost the defence more runs than the current model, not fewer.

## 4. What the wall costs a fielder (Statcast)

SOURCED, MLB.com 2018 article above:
- A "wall ball" = projected to land within 8 ft of an outfield wall (not side walls) and no higher than 8 ft up. Wall balls are compared only against each other.
- About 2,000 of nearly 33,000 tracked catches in 2017 (about 6 %) were wall balls. Only 64 moved by 40 or more points of Catch Probability.
- Examples, old to new Catch Probability: Pillar 41 % to 2 %; Stanton 55 % to 11 %; Bradley Jr. 69 % to 24 %.
- Published only as before/after examples; no average penalty in percentage points or runs, and no seconds-of-delay figure. The adjustment is a statistical regression on outcomes, so it already mixes braking, leaping, and fear of injury and cannot be turned into an arrival speed.
- Later MLB text says the expected catch rate uses distance, time, and in some cases direction and wall (https://www.mlb.com/news/outfield-defense-is-better-than-ever-in-2024).

## 5. Conclusion for the lead

- **Arrival speed at contact: no sourced number exists.** Do not cite 2.5 m/s as sourced (the reviewer's figure has no source, and I found none either).
- What the evidence does support: (a) fielders run near top speed (7.6 to 8.9 m/s peaks on highlight wall plays) until close to the wall; (b) they do brake, because the track and padding exist and collisions are still notable injuries; (c) full stop at the wall is not what real fielders do on catches, since a leap or hand-on-wall catch means non-zero contact speed, but how non-zero is unmeasured.
- ESTIMATE, low confidence: real contact speeds on attempted wall catches are somewhere between a few tenths and a few m/s, with the high tail on the plays that become highlights and injuries. The sim's current "about 0" is the low edge; 2.5 m/s sits inside the plausible range but is unsupported. I would not claim better than "somewhere in 0 to 4 m/s".
- Braking rate: 6 m/s² is above the measured team-sport mean (4.2 to 5.0 m/s²). Sourced range supports 4.5 to 5 as a default if the sim wants a measured value.
- Named parameters the sim would need, if the owner wants this modelled rather than fixed:
  - `WALL_CONTACT_SPEED_MPS` (speed toward the wall when the fielder's body reaches it; today implicitly 0). The cap would become `sqrt(v_c² + 2 * BRAKE * dist)`.
  - `BRAKE_MPS2` (4.5 to 5 sourced; 6 current).
  - A commit-to-brake trigger: the distance or time at which the fielder starts easing off, with a perception noise term (he cannot see the wall while watching the ball), and a per-fielder attribute (wall awareness or fearlessness) driving both. This is where an imperfect-but-believable mistake belongs: a fielder who misjudges the wall distance arrives hard.
  - Optional: leap or hand-brace outcome for balls within about 2.4 to 3 m (the sim's `catch_ceiling` region).
- How to get real numbers if wanted: hand-measure arrival speed from broadcast video of 20 to 30 wall catches (frame rate and distance to wall known from park dimensions); or ask MLB/Baseball Savant for a research data request. Both are outside this note.

## Licence and reuse

- Statcast figures and MLB.com text: copyright MLB Advanced Media; citing numbers is fine, bulk reuse of Savant data is restricted by MLB terms (I did not re-read them; check before any scrape or redistribution).
- Harper et al. 2020 is an MDPI open-access paper (journal policy is CC BY; confirm on the article page). Citing the numbers is fine.
- Not used: Driveline OpenBiomechanics (licence excludes commercial use).
- Items marked "search snippet, unverified" must be re-opened before they go into a spec.
