# Review: client fixes from the owner's seed 51001 saved plays (`work/client-51001`, `0d3fddd..499393b`)

Reviewer: Sonnet (wrote none of the change). Client-only diff; no sim change, no ledger row needed.

## Findings

### P1 — The fair signal fires on routine balls, because "near the line" includes the bounce near home plate
`client/Source/BaseballClient/BbDirector.cpp` `NoteSignals`, the new fair branch (~L2262–2330): `Closest` is the minimum of `Inside(P)` over **every sampled ground point from the first bounce on**, and the test is `Closest <= FairNearLineM` (4 m). Near the plate every point is within 4 m of the lines (the lines diverge from the apex: a ball landing at y = 3–5 m up the middle is already 2–3.5 m "from the line"). So any grounder that first lands within about 5 m of the plate counts as "near a line" for its whole life, and the signal is then placed where the ball was finally decided (`At`), which may be 20 m from any line.
- Seen with `-BbStagingLog` on the owner's own saved plays: play 66 "Ground out to short" gets `fair` at (-6.5, 32.4) (18 m from the line); play 102 "Single to center" gets `fair` at (5.1, 34.3) (20 m from the line). Both are routine; player-visuals §12 says routine ground balls get none, and the owner asked for the signal on a ball "down the line".
- Suggested fix: judge nearness only where the ball is decided (`Inside(At)` at the decision, plus the point where it passes or lands beyond the bags), or for a ground ball take the minimum only over samples with `Along(P) >= BagM - 3` (the plate umpire's own ground) — better, use an angle from the line rather than metres so the near-plate region does not count. Add a probe or test over these plays (66, 102 must give none; 89, 146 must still give one).

### P2 — The between-innings card shows an extra, not-yet-batted inning for the side that bats next
Seen in the worker's own shot `client/Saved/Shots/shot_21.png` and re-read by me: "END OF THE 2ND" (▼ 2 on the bug) shows **three** columns for the Colliers (0 0 0) and two for the Larks; the same cause gives the two innings at "END OF THE 1ST" the worker noted. `BbSim::BoxScore(Through)` is read through the last play shown, and `bb_stats` pushes a linescore slot on `HalfInningStarted` (`sim/crates/bb_stats/src/lib.rs:165-167`), which the rules engine emits in the play that makes the third out. So the box already holds the next half's empty slot, and the card prints "0" for an inning not played, against hud.md "an inning not batted yet is empty". The change at the card (`499393b`) did not cause this (it is the step-4 code in `BbDirector.cpp` ~L4378 and `DrawInningCard`), so it is older; the 10 s hold now makes it readable. The right changeover is shown (END after the bottom, MIDDLE after the top, via `Before`), the runs and hits are right.
- Suggested fix: in `DrawInningCard` (or when `LineBox` is copied for the card) drop the last slot of the side that bats next when its runs and hits are 0 and no play of that half has been shown (`n_innings` of the side against `Before.inning`/`Before.top`). Same for the park scoreboard if it prints the slot.

### P3 — "Enter: continue" sits at the bottom edge next to the play-by-play banner, not under the card
`BbGameMode.cpp:~89`: drawn at `ClipY*0.5 + 130*S`. In `shot_21.png` it is at y≈442 of 500, directly over the call banner "Hayashi: Ground out to the pitcher", 130 px below the card's bottom edge. Readable but looks like part of the banner. Fix: place it under the card's own box (card bottom + a margin).

### P3 — Stale measurement line in player-visuals §12
"Measured (`-BbStagingLog`, seed 7, 94 pitches): 59 signals: 33 strikes, 17 outs, 7 fouls, 2 safes" predates `fair`; re-measure or mark it as before the fair signal.

### P3 — Camera cut lags contact by 1–2 frames
`BB_INPLAY` logs: play 112 cut at 0.429 for contact 0.401; play 89 0.427/0.392; play 146 0.420/0.393; play 128 0.428/0.401 (the added `Frame.ball[1] > 1.5` condition). Never before contact, so the owner's complaint is fixed; noting it only so nobody tightens the margin below the ball's 1.5 m of travel.

## Answers to the questions

1. **Fair signal.** Timing and mechanics are right: the decision comes from the drawn ball path (`SampleDrawn`), after the OBR event (landing at or beyond the bag, passing 1B/3B after touching ground, first touched off the ground by a fielder, settling in the infield, over the fence in flight via `dead_reason` 2); a ball taken in the air breaks out with no signal (play 128 line out to center: none; no signal on a foul, which goes down the existing foul branch because `pitch_call == 6` is the fair-in-play test). Umpire: plate umpire if `Along < BagM-3`, else the first-base umpire for X > 0 and the third-base umpire for X < 0; plays 89 and 146 give umpire 3 (third), `fair` at t=1.43 and 1.33 (contact 0.39), 0.15 s after the ball comes down at the third-base bag. Arm: the infield-side arm, pointing toward fair ground. The failure is the nearness test (P1), not the timing. I did not find a case of a home run or foul ball receiving it (home runs only count if the clearing point is within 4 m of the pole line, which is reasonable).
2. **Camera and other pre-contact cues.** Fixed (`BB_INPLAY` after contact in 4 plays). Read `BbDirector.cpp`: the pitch line gates on `Arrived = contact_s` (L4451), the catcher's mitt/crouch, `Ct.bSet`, and `ReadyUntil` on `contact_s`, outs on their call times, signals on their own times. I did **not** check fielders' and runners' first steps in the drawn timeline frames; those come straight from the sim's timeline and I made no test that they start after `contact_s`.
3. **Card.** 10 s of real time, Enter skips, speed keys and the rush cannot shorten it: from the code, `SideCardHeldS += RealDt` (unscaled, not while paused), both staging clocks are held with `ResetTime -= SimDt`, the hold ends on Enter or the time, then the fade-in runs from `BackAt`. The staged `Enter` also sets `bRush`, so the next pitch is rushed as a deliberate "continue". Right changeover and line score except the extra empty slot (P2, older than this change).
4. **Docs.** player-visuals §12, hud.md and unreal-spike.md describe the change accurately, except that §12 promises "routine ground balls … get no signal" (P1 contradicts it) and carries the stale measurement (P3).

## Verification

- Built the client: `taskset -c 0-7 nice Build.sh BaseballClientEditor … -MaxParallelActions=2` after touching `BbDirector.cpp` and `BbCrew.cpp` (succeeded, 15 s). One client run at a time.
- Loaded saved plays 112, 89, 128, 146, 66, 168, 102, 71, 90 (`-RenderOffscreen -BbSavedPlay=… -BbStagingLog`, each killed after 40–45 s since a saved play does not exit) and read `BB_SIGNAL` and `BB_INPLAY` from `client/Saved/Logs/BaseballClient.log`: 89 fair (umpire 3, t=1.43), 146 fair (umpire 3, t=1.33), 128 none; 66 fair at (-6.5, 32.4) and 102 fair at (5.1, 34.3) (P1); 168, 71, 90 no fair signal (71 is a long triple to right: none).
- `-BbCrewShots` (marker `…/scratchpad/m2`, shots newer than it): read `crew_11_fair.png`: the first-base umpire stands foul side of the line, raises the arm toward the infield (fair) side, other arm hanging; correct arm and direction.
- Read the worker's `client/Saved/Shots/shot_21.png` (card "END OF THE 2ND" with three Colliers columns and two Larks, "Enter: continue" at the bottom) and `shot_03.png` (a normal pitch view, no card).
- Not run: the full game card at 10x with `-BbCardEnterAt`, the cargo tests (client-only diff), the fielders' first-step timing.

## Resolution

Open.
