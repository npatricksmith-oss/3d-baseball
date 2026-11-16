# HUD: game graphics

**Status:** plan agreed with the owner 2026-11-15 (three questions, Q22–Q24, all as recommended; §1). **All four steps built 2026-11-16** (§5); the HUD plan is complete. Third part of the graphics stage, after [the park](park-visuals.md) and [the players](player-visuals.md).

**Where it starts.** `ABbHUD::DrawHUD` (`BbGameMode.cpp`) draws a dark strip with one line of score text, one line about the last pitch, a three-square runners diamond, a footer of keys, the between-innings card and the home-run card. All of it is the engine's large font on the canvas.

## 1. Decisions (owner, 2026-11-15)

| # | Question | Ruling |
|---|---|---|
| Q22 | Style | **A broadcast-style score bug** in a corner: clubs, score, inning and half, the diamond, count, outs. Drawn in code. |
| Q23 | What else shows | **The full set, built in this order:** the bug and the play caption; a batter card when he steps in (name, number, position, his day so far); the pitch's speed and type after each pitch; the pitcher's pitch count; a strike-zone box with the pitch's location, on a key; a line-score card between innings; a full box score at the end. |
| Q24 | Lettering | **The engine's built-in font** for the HUD. The dot alphabet stays on the scoreboard. |

Carried over from the park plan: the lead builds alone in short sessions, shows screenshots after each merge, no Codex reviews, nothing binary committed.

## 2. Approach

- **Everything shown is read from the sim,** never counted in the client: the score, the line score and the batters' lines from `bb_game_box_score` (statistics are functions of the event log, invariant 6); names, numbers and positions from `bb_game_roster`; the pitch's speed, type and plate location from the timeline. If a figure the HUD needs is not exported (the pitcher's pitch count, the pitch's type), it is added as a read-only export under the players plan's rule (Q21): digests unchanged, one ledger line.
- **A HUD apart from the game mode.** New `BbHud.{h,cpp}` holds the panels; each panel is a small function over one plain struct of what to show, so the director fills structs and never draws.
- **One place for the look.** Panel colours, sizes and the clubs' colours come from `BbLook.h`. The clubs' names and short forms come from the players plan (§5 there).
- **Canvas drawing,** scaled from a 720-line layout as now. No widget assets.
- **The scoreboard in the park keeps working** and must agree with the bug: both read the same box score.

## 3. Steps

1. **The bug and the caption.** Replaces the strip. The home-run card and the between-innings card are restyled to match.
2. **The batter card and the pitch readout.** Card on stepping in, gone at the first pitch; speed and type after each pitch; the pitch count beside the pitcher's name.
3. **The strike-zone box.** The batter's own zone from the sim, each pitch of the at-bat marked where it crossed, called strikes and balls told apart; a key turns it on and off.
4. **Line score between innings; box score at the end.**

## 4. Checking the look

- `client/run.sh 7 -RenderOffscreen -BbShots=6`, read at 720 lines and at 1080.
- A flag that holds each panel on screen for a still, so a panel that shows for two seconds can be photographed without hunting for the moment.
- The bug must not cover the ball's usual paths from the broadcast camera.

## 5. Built

**Step 1, the bug and the caption (2026-11-16).** `BbHud.{h,cpp}` draws the panels; `ABbHUD` (`BbGameMode.cpp`) only places them, and still draws the box score itself (step 4 will move it). Colours are `BbLook::Hud*`, given in sRGB.
- **The bug,** top left, 292 × 64 reference units: each club's row with its colour down the edge ("COLLIERS" visitors above, "LARKS" home below; the trims lifted, since the uniforms' navy and dark red are near black on a dark panel), the runs in a lighter box, the bases as three diamonds lit gold, the outs as three squares under them, and the inning with an up or down arrow over the count. Filled from `FBbBugInfo`, which the director sets from the same play record as before (runs, count and bases when the play is known, outs as the umpire calls them). Under it, slim lines for the pitch's speed (exit speed once hit), the replay tag in gold and a saved-play note.
- **The caption,** low and centred above the keys, edged in the batting side's colour: the sim's summary of the play, at the same moment as before. It was at the top centre, over the field.
- **Above the caption,** one at a time: the umpire's out signal (edged white, or red with red letters on a bang-bang play) or the home-run card (edged gold, the hitter's club down its side: what it scored, the name, distance and speed).
- **Between innings:** "MIDDLE OF THE 3RD" over both clubs' full names and runs, on the black.
- **`-BbHudHold`** keeps every panel on screen, with made-up words where the game has none yet (hud.md §4). Use it with `-BbShots`. `-ForceRes` makes the offscreen shots 1600 × 900; without it they come out 888 × 500.
- Pictures: `client/Saved/Shots/hud1_before/`, `hud1_after/`, `hud1_hold/` (untracked).
- **For the owner to overrule if he likes:** the clubs by nickname on the bug; the caption at the bottom; three out marks, not two.

**Step 2, the pitcher, the batter and the pitch (2026-11-16).** The owner asked, while it was being built, for the names on the bug itself: so instead of a batter card that comes and goes, **two rows under the score stay up** (owner, 2026-11-16).
- **The pitcher's row,** in the fielding club's colour: "P", his number and name, and the pitches he has thrown ("42 P"). The count goes up as the pitch leaves his hand.
- **The batter's row,** in the batting club's colour: his position, number and name, and his game so far: hits for at-bats, then home runs, triples, doubles, runs batted in, walks and strikeouts ("1-3, HR, 2 RBI"), or "first time up". It changes when the result is known, not before.
- **The pitch readout** under them: speed and type ("87 mph Slider") once the pitch reaches the plate or the bat, not before (it used to show the speed during the wind-up); then the ball's speed and angle off the bat.
- **Read from the sim:** names, numbers and positions from the box score's lines; the pitch count from new pitcher lines in the box score (`bb_stats::PitcherLine`, counted from `PitchReleased`, so a function of the event log; `BbBoxScore.pitchers`); the pitch type from the timeline's telemetry (`Telemetry.pitch_type`, the type the log records as thrown; `BbPlayInfo.pitch_type`, in a byte that was padding). Game digests unchanged; test `pitch_type_and_pitch_count_agree_with_what_the_client_was_handed` (bb_ffi).
- **Speed keys (owner, same day):** 1 to 5 jump to 1x, 2x, 4x, 6x and 10x; minus slows down by 0.25x at a time (down to 0.25x), equals speeds up the same way (up to 10x). The middle steps are the lead's choice; the owner asked only for 1x to 10x.
- Pictures: `client/Saved/Shots/hud2_after/` (untracked).

**Step 3, the strike-zone box (2026-11-16).** Top right, under the same panel style: this batter's zone as the sim judges it (his own height, the 2026 ABS fractions, the plate's width), with its thirds marked faintly, and every pitch of the at-bat centred where it crossed the middle of the plate, drawn half again as large as the ball so its number reads (so a mark can overlap the edge on a pitch called a ball), numbered in order, the newest ringed in white. Green a ball, red a called strike, gold a swing or a foul, blue in play (grey a hit batter); a key underneath says so. **Seen from the pitcher's side**, as the centre-field camera sees it: the first-base side is on the left. A pitch goes in when it reaches the plate. **Z** hides and shows it; it is up by default and hidden under the between-innings card.
- **Read from the sim:** the crossing point, the zone and the call, from the timeline's telemetry (`Telemetry.zone_bottom_m`, `zone_top_m`, `zone_half_width_m`, `plate_crossed`, `pitch_call`, beside the `plate_x_m`/`plate_z_m` that were there) into `BbPlayInfo` (appended; the struct grew). The zone is carried on `PitchRecord.zone`. The at-bat's earlier pitches come from the client's play history, so a replay shows the at-bat as it stood. Digests unchanged; test `a_taken_pitch_is_called_by_where_it_crossed_the_exported_zone` (bb_ffi) checks every taken pitch's call against the ball touching the exported zone.
- Pictures: `client/Saved/Shots/hud3_after/` (untracked).
- **For the owner to overrule if he likes:** the pitcher's-side view (the umpire's view flips it), the colours, the box on by default.

**Step 4, the line score and the box score (2026-11-16).** The last step.
- **Between innings,** the card on the black is now the line score: "MIDDLE OF THE 3RD" over a row per club, an inning to a column (nine at least, more and narrower in extra innings), then runs and hits. An inning not batted yet is empty. It reads the same box score as the scoreboard in the park (`ABbHUD::LineBox`, copied when the director refreshes the board once a pitch), so the two agree. If the sim hands over no box score the card falls back to the two clubs' runs.
- **The box score** (Tab during the game, and at the end over black) is in the HUD's look instead of the newspaper page: the score in words with the clubs' nicknames, the winner first at the end ("LARKS 3, COLLIERS 0", "FINAL - 11 INNINGS"); the same line score; each club's batting lines under its full name and colour, with the men left on; under each table the doubles, triples, home runs, sacrifice flies and hit batters, and the club's pitchers with the pitches each threw.
- **Moved:** the box is drawn by `BbHud::DrawBoxScore` (it was `ABbHUD::DrawBox` in `BbGameMode.cpp`); the game mode's HUD now only places panels. Client only: no sim change, no ledger row.
- **Checks:** `-BbBoxShots=<n>` photographs the Tab box at pitch n and the final box (`box_midgame.png`, `box_final.png`); `-BbHudHold` shows the card.
- Pictures: `client/Saved/Shots/hud4_after/` (untracked).
- **Not shown, because the sim does not export them:** errors in the line score (R and H only), and the pitchers' innings, hits, runs and strikeouts (pitches only). Both would be read-only exports from the event log.
- **For the owner to overrule if he likes:** the dark panel in place of the newspaper page; nicknames in the title and the line score.
